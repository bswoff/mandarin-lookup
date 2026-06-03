use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchDefinition {
    pub sense_label: String,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchEntry {
    pub character: String,
    pub pinyin: String,      // tone marks for display, e.g. "guò"
    pub pinyin_key: String,  // no tones, no spaces — used in URLs, e.g. "guo"
    pub definitions: Vec<SearchDefinition>,
}

pub struct CedictIndex {
    by_pinyin: HashMap<String, Vec<IndexedEntry>>,
    by_char: HashMap<String, Vec<IndexedEntry>>,
}

#[derive(Clone)]
struct IndexedEntry {
    traditional: String,
    pinyin_display: String,
    pinyin_key: String,
    definitions: Vec<String>,
}

impl CedictIndex {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        Self::from_reader(BufReader::new(file))
    }

    fn from_reader<R: std::io::BufRead>(reader: R) -> Result<Self> {
        let mut by_pinyin: HashMap<String, Vec<IndexedEntry>> = HashMap::new();
        let mut by_char: HashMap<String, Vec<IndexedEntry>> = HashMap::new();

        for entry in cedict::parse_reader(reader) {
            let traditional = entry.traditional().to_string();
            let raw_pinyin = entry.pinyin().to_string();

            let definitions: Vec<String> = entry
                .definitions()
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty() && !is_junk_definition(d))
                .collect();

            if definitions.is_empty() {
                continue;
            }

            let pinyin_display = convert_pinyin(&raw_pinyin);
            let pinyin_key = normalize_cedict_pinyin(&raw_pinyin);

            let indexed = IndexedEntry {
                traditional: traditional.clone(),
                pinyin_display,
                pinyin_key: pinyin_key.clone(),
                definitions,
            };

            by_pinyin.entry(pinyin_key).or_default().push(indexed.clone());
            by_char.entry(traditional).or_default().push(indexed);
        }

        Ok(Self { by_pinyin, by_char })
    }

    pub fn lookup_by_pinyin(&self, pinyin_base: &str) -> Vec<SearchEntry> {
        match self.by_pinyin.get(pinyin_base) {
            Some(entries) => {
                let mut results = entries_to_search_entries(entries);
                // More definitions is a proxy for a more common or versatile word; surface those first.
                results.sort_by(|a, b| b.definitions.len().cmp(&a.definitions.len()));
                results
            }
            None => vec![],
        }
    }

    pub fn lookup_by_char(&self, character: &str) -> Vec<SearchEntry> {
        match self.by_char.get(character) {
            Some(entries) => {
                let mut results = entries_to_search_entries(entries);
                // More definitions is a proxy for a more common or versatile word; surface those first.
                results.sort_by(|a, b| b.definitions.len().cmp(&a.definitions.len()));
                results
            }
            None => vec![],
        }
    }

    pub fn lookup_by_english(&self, query: &str) -> Vec<SearchEntry> {
        let query_words: Vec<String> = query
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();

        if query_words.is_empty() {
            return vec![];
        }

        // Ranking tiers (best first):
        //   1. exact_on_first  — definition[0] == query exactly (case-insensitive)
        //   2. exact_any       — any definition == query exactly
        //   3. first_match_pos — position of first definition containing all query words (lower = better)
        //   4. match ratio     — match_count / total_defs (higher = better, cross-multiplied to avoid floats)
        let query_exact = query.trim().to_lowercase();

        let mut results: Vec<(IndexedEntry, bool, bool, usize, usize, usize)> = self
            .by_pinyin
            .values()
            .flatten()
            .filter_map(|entry| {
                let mut first_pos = None;
                let mut match_count = 0usize;
                let mut exact_any = false;

                for (i, def) in entry.definitions.iter().enumerate() {
                    if def.to_lowercase() == query_exact {
                        exact_any = true;
                    }
                    if query_words.iter().all(|qw| contains_word(def, qw)) {
                        if first_pos.is_none() {
                            first_pos = Some(i);
                        }
                        match_count += 1;
                    }
                }

                let exact_on_first = entry.definitions.first()
                    .map(|d| d.to_lowercase() == query_exact)
                    .unwrap_or(false);

                first_pos.map(|pos| (
                    entry.clone(),
                    exact_on_first,
                    exact_any,
                    pos,
                    match_count,
                    entry.definitions.len(),
                ))
            })
            .collect();

        results.sort_by(|a, b| {
            b.1.cmp(&a.1)                           // exact_on_first: true first
                .then_with(|| b.2.cmp(&a.2))        // exact_any: true first
                .then_with(|| a.3.cmp(&b.3))        // first_match_pos: lower first
                .then_with(|| (b.4 * a.5).cmp(&(a.4 * b.5))) // match ratio: higher first
        });
        results.truncate(20);
        let entries: Vec<IndexedEntry> = results.into_iter().map(|(e, ..)| e).collect();
        entries_to_search_entries(&entries)
    }

    /// Given a pinyin_key (e.g. "guo") and a slug (e.g. "to-cross"), returns the
    /// matching (traditional character, definition text, pinyin_display) triple from CEDICT, or None.
    pub fn resolve_entry(&self, pinyin_key: &str, slug: &str) -> Option<(String, String, String)> {
        let entries = self.by_pinyin.get(pinyin_key)?;
        for entry in entries {
            for def in &entry.definitions {
                if make_slug(def) == slug {
                    return Some((entry.traditional.clone(), def.clone(), entry.pinyin_display.clone()));
                }
            }
        }
        None
    }
}

fn entries_to_search_entries(entries: &[IndexedEntry]) -> Vec<SearchEntry> {
    let result: Vec<SearchEntry> = entries
        .iter()
        .map(|entry| {
            let mut slug_counts: HashMap<String, u32> = HashMap::new();
            let definitions = entry
                .definitions
                .iter()
                .map(|def| {
                    let base_slug = make_slug(def);
                    let count = slug_counts.entry(base_slug.clone()).or_insert(0);
                    *count += 1;
                    let slug = if *count == 1 {
                        base_slug
                    } else {
                        format!("{}-{}", base_slug, count)
                    };
                    SearchDefinition {
                        sense_label: def.clone(),
                        slug,
                    }
                })
                .collect();

            SearchEntry {
                character: entry.traditional.clone(),
                pinyin: entry.pinyin_display.clone(),
                pinyin_key: entry.pinyin_key.clone(),
                definitions,
            }
        })
        .collect();

    result
}

// Converts the full definition text to a URL-safe slug.
// Runs of non-alphanumeric ASCII chars become a single hyphen; leading/trailing hyphens stripped.
// e.g. "to pass (time)" → "to-pass-time", "too-" → "too"
fn make_slug(sense_label: &str) -> String {
    let mut result = String::new();
    let mut prev_hyphen = false;

    for c in sense_label.chars() {
        if c.is_ascii_alphanumeric() {
            result.push(c.to_ascii_lowercase());
            prev_hyphen = false;
        } else if !result.is_empty() && !prev_hyphen {
            result.push('-');
            prev_hyphen = true;
        }
    }

    result.trim_end_matches('-').to_string()
}

fn contains_word(text: &str, word: &str) -> bool {
    let text_lower = text.to_lowercase();
    let mut start = 0;
    while start < text_lower.len() {
        match text_lower[start..].find(word) {
            None => return false,
            Some(rel) => {
                let abs = start + rel;
                let before_ok = abs == 0
                    || !text_lower.as_bytes()[abs - 1].is_ascii_alphanumeric();
                let end = abs + word.len();
                let after_ok = end >= text_lower.len()
                    || !text_lower.as_bytes()[end].is_ascii_alphanumeric();
                if before_ok && after_ok {
                    return true;
                }
                start = abs + 1;
            }
        }
    }
    false
}

fn is_junk_definition(def: &str) -> bool {
    def.starts_with("CL:") || def.contains("old variant")
}

// Strips tone digits and spaces from raw CEDICT pinyin, e.g. "guo4" → "guo", "xiu1 xi5" → "xiuxi"
pub fn normalize_cedict_pinyin(raw: &str) -> String {
    raw.split_whitespace()
        .map(|syl| {
            syl.trim_end_matches(|c: char| c.is_ascii_digit())
                .to_lowercase()
                .replace("u:", "u")
        })
        .collect::<String>()
}

// Converts CEDICT number-tone pinyin to tone-mark pinyin, e.g. "guo4" → "guò"
pub fn convert_pinyin(raw: &str) -> String {
    raw.split_whitespace()
        .map(|syl| apply_tone(syl))
        .collect::<String>()
}

fn apply_tone(syllable: &str) -> String {
    let lower = syllable.to_lowercase();

    let last_char = lower.chars().last();
    let (body, tone) = if last_char.map_or(false, |c| c.is_ascii_digit()) {
        let tone = last_char.unwrap().to_digit(10).unwrap_or(5) as usize;
        // safe: last char is ASCII digit, so exactly 1 byte
        (lower[..lower.len() - 1].to_string(), tone)
    } else {
        (lower.clone(), 5)
    };

    // CEDICT encodes ü as "u:"
    let body = body.replace("u:", "ü");

    if tone == 0 || tone == 5 {
        return body;
    }

    let chars: Vec<char> = body.chars().collect();
    match find_tone_position(&chars) {
        Some(idx) => chars
            .iter()
            .enumerate()
            .map(|(i, &c)| if i == idx { apply_tone_mark(c, tone) } else { c })
            .collect(),
        None => body,
    }
}

// Pinyin tone placement rules:
// 1. 'a' or 'e' always takes the mark
// 2. 'ou' → mark goes on 'o'
// 3. Otherwise mark the last vowel
fn find_tone_position(chars: &[char]) -> Option<usize> {
    if let Some(i) = chars.iter().position(|&c| c == 'a' || c == 'e') {
        return Some(i);
    }
    for (i, window) in chars.windows(2).enumerate() {
        if window[0] == 'o' && window[1] == 'u' {
            return Some(i);
        }
    }
    chars
        .iter()
        .rposition(|&c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'ü'))
}

fn apply_tone_mark(c: char, tone: usize) -> char {
    match (c, tone) {
        ('a', 1) => 'ā', ('a', 2) => 'á', ('a', 3) => 'ǎ', ('a', 4) => 'à',
        ('e', 1) => 'ē', ('e', 2) => 'é', ('e', 3) => 'ě', ('e', 4) => 'è',
        ('i', 1) => 'ī', ('i', 2) => 'í', ('i', 3) => 'ǐ', ('i', 4) => 'ì',
        ('o', 1) => 'ō', ('o', 2) => 'ó', ('o', 3) => 'ǒ', ('o', 4) => 'ò',
        ('u', 1) => 'ū', ('u', 2) => 'ú', ('u', 3) => 'ǔ', ('u', 4) => 'ù',
        ('ü', 1) => 'ǖ', ('ü', 2) => 'ǘ', ('ü', 3) => 'ǚ', ('ü', 4) => 'ǜ',
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn index_from_str(cedict_text: &str) -> CedictIndex {
        CedictIndex::from_reader(BufReader::new(Cursor::new(cedict_text.as_bytes())))
            .expect("test CEDICT data should parse")
    }

    // Isolates the sort key logic with plain tuples — no CEDICT parsing involved.
    // Tuple layout mirrors lookup_by_english: (name, exact_on_first, exact_any, first_pos, match_count, total_defs)
    #[test]
    fn english_search_no_partial_word_match() {
        // "xx" must not match "x", "xxx", or "axxb" (embedded substring).
        // It must match "xx", "xx yy" (space boundary), and "yy (xx)" (paren boundary).
        let cedict_text = "\
甲 甲 [jia3] /x/
乙 乙 [yi3] /xxx/
丙 丙 [bing3] /axxb/
丁 丁 [ding1] /xx/
戊 戊 [wu4] /xx yy/
己 己 [ji3] /yy (xx)/
";
        let index = index_from_str(cedict_text);
        let results = index.lookup_by_english("xx");
        let mut chars: Vec<&str> = results.iter().map(|r| r.character.as_str()).collect();
        chars.sort(); // HashMap order is non-deterministic; sort for stable assertion

        assert_eq!(chars, vec!["丁", "己", "戊"], "whole-word matches only; spaces and parens are valid boundaries");
    }

    #[test]
    fn character_search_no_substring_match() {
        // Searching for 過 must not return entries whose character contains 過 as a component.
        let cedict_text = "\
過 过 [guo4] /to pass/to go through/
書過 书过 [shu1 guo4] /to write past/
";
        let index = index_from_str(cedict_text);
        let results = index.lookup_by_char("過");
        let chars: Vec<&str> = results.iter().map(|r| r.character.as_str()).collect();

        assert_eq!(chars, vec!["過"], "只 過 should match, not 書過");
    }

    #[test]
    fn pinyin_search_no_substring_match() {
        // Searching for "man" must not return entries whose pinyin_key merely contains "man"
        // (e.g. "mang" or "manren") — only exact key matches.
        let cedict_text = "\
慢 慢 [man4] /slow/
忙 忙 [mang2] /busy/
漫人 漫人 [man4 ren2] /someone/
";
        let index = index_from_str(cedict_text);
        let results = index.lookup_by_pinyin("man");
        let chars: Vec<&str> = results.iter().map(|r| r.character.as_str()).collect();

        assert_eq!(chars, vec!["慢"], "only exact pinyin_key match should be returned");
    }

    #[test]
    fn sort_key_isolated() {
        let mut data: Vec<(&str, bool, bool, usize, usize, usize)> = vec![
            ("蹇",   false, true,  3, 1, 8),
            ("遲慢",  true,  true,  0, 1, 2),
            ("慢",   true,  true,  0, 1, 1),
            ("緩",   true,  true,  0, 1, 12),
        ];
        data.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| b.2.cmp(&a.2))
                .then_with(|| a.3.cmp(&b.3))
                .then_with(|| (b.4 * a.5).cmp(&(a.4 * b.5)))
        });
        let names: Vec<&str> = data.iter().map(|d| d.0).collect();
        assert_eq!(names, vec!["慢", "遲慢", "緩", "蹇"]);
    }

    #[test]
    fn english_search_slow_ranking() {
        // Real CEDICT entries that contain "slow" as a whole word in at least one definition.
        // Expected ranking:
        //   1. 慢  — exact_on_first=true, ratio 1/1  (highest)
        //   2. 遲慢 — exact_on_first=true, ratio 1/2
        //   3. 緩  — exact_on_first=true, ratio 1/12 (lowest exact-first)
        //   4. 蹇  — exact_on_first=false, exact_any=true ("slow" at position 3)
        let cedict_text = "\
慢 慢 [man4] /slow/
蹇 蹇 [jian3] /lame/cripple/unfortunate/slow/difficult/nag (inferior horse)/donkey/lame horse/
緩 缓 [huan3] /slow/unhurried/sluggish/gradual/not tense/relaxed/to postpone/to defer/to stall/to stave off/to revive/to recuperate/
遲慢 迟慢 [chi2 man4] /slow/late/
";
        let index = index_from_str(cedict_text);
        let results = index.lookup_by_english("slow");
        let chars: Vec<&str> = results.iter().map(|r| r.character.as_str()).collect();

        assert_eq!(results.len(), 4, "all 4 entries should match");
        assert_eq!(chars[0], "慢",  "慢 should rank 1st: exact_on_first, ratio 1/1");
        assert_eq!(chars[1], "遲慢", "遲慢 should rank 2nd: exact_on_first, ratio 1/2");
        assert_eq!(chars[2], "緩",  "緩 should rank 3rd: exact_on_first, ratio 1/12");
        assert_eq!(chars[3], "蹇",  "蹇 should rank 4th: exact_any but not first");
    }

    #[test]
    fn pinyin_search_sorts_by_definition_count() {
        // Three dummy entries sharing pinyin "xx1" with distinct definition counts.
        // Content of definitions is irrelevant — only the count drives the sort.
        let cedict_text = "\
甲 甲 [xx1] /a/
乙 乙 [xx1] /a/b/c/
丙 丙 [xx1] /a/b/
";
        let index = index_from_str(cedict_text);
        let results = index.lookup_by_pinyin("xx");
        let chars: Vec<&str> = results.iter().map(|r| r.character.as_str()).collect();

        assert_eq!(results.len(), 3);
        assert_eq!(chars[0], "乙", "3 definitions → 1st");
        assert_eq!(chars[1], "丙", "2 definitions → 2nd");
        assert_eq!(chars[2], "甲", "1 definition  → 3rd");
    }

    #[test]
    fn character_search_sorts_by_definition_count() {
        // Same character 字, three different pronunciations with distinct definition counts.
        // Content of definitions is irrelevant — only the count drives the sort.
        // Assert on pinyin_key (tone-stripped) to avoid tone-mark conversion details.
        let cedict_text = "\
字 字 [ma1] /a/
字 字 [ta1] /a/b/
字 字 [ge1] /a/b/c/
";
        let index = index_from_str(cedict_text);
        let results = index.lookup_by_char("字");
        let keys: Vec<&str> = results.iter().map(|r| r.pinyin_key.as_str()).collect();

        assert_eq!(results.len(), 3);
        assert_eq!(keys[0], "ge", "3 definitions → 1st");
        assert_eq!(keys[1], "ta", "2 definitions → 2nd");
        assert_eq!(keys[2], "ma", "1 definition  → 3rd");
    }
}
