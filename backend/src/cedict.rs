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
        let reader = BufReader::new(file);

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
            Some(entries) => entries_to_search_entries(entries),
            None => vec![],
        }
    }

    pub fn lookup_by_char(&self, character: &str) -> Vec<SearchEntry> {
        match self.by_char.get(character) {
            Some(entries) => entries_to_search_entries(entries),
            None => vec![],
        }
    }
}

fn entries_to_search_entries(entries: &[IndexedEntry]) -> Vec<SearchEntry> {
    let mut result: Vec<SearchEntry> = entries
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

    result.sort_by(|a, b| b.definitions.len().cmp(&a.definitions.len()));
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
