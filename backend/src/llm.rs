use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const TIER1_MODEL: &str = "claude-haiku-4-5";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SenseCandidate {
    pub character: String,
    pub pinyin: String,
    pub sense_label: String,
    pub slug: String,
}

const TIER1_SYSTEM: &str = "You are a Mandarin Chinese dictionary. Given a query, return every distinct sense as a JSON array.

INPUT HANDLING — follow exactly based on what the input is:

1. Single Chinese character: return all distinct senses of that character.

2. Multiple Chinese characters (a compound word): treat the entire input as one compound word. Return senses for the compound as a whole. NEVER decompose a compound into its individual characters. The `character` field must be the full compound string exactly as given (e.g. given '休息', character must be '休息', never '休' or '息' separately). The `pinyin` field must be the full compound reading (e.g. 'xiūxi', never 'xiū' alone).

3. Pinyin (ASCII letters only): return all characters across all tones whose base syllable — with tone marks stripped — exactly matches the input. NEVER return characters whose base syllable only partially matches (e.g. input 'guo' must not return characters for 'gu', 'ge', or any other syllable). Each character-sense pair is its own object.

OUTPUT RULES — follow every rule exactly:

- Return ONLY a valid JSON array. No preamble, no explanation, no markdown code fences.
- Each array element is one distinct grammatical or semantic sense. Do not list nuanced synonyms as separate senses.
- Use Traditional Chinese characters throughout.
- `sense_label`: a short English phrase, 2–4 words. Use 'to [verb]' for actions, or a single clear noun phrase for other senses. NEVER join alternatives with 'or', 'and', commas, slashes, or parentheses — if a word has two distinct meanings, give each its own element with its own distinct label.
- `slug`: always take exactly the first 2 words of `sense_label` (or the full label if it is only 1 word), lowercased, with spaces replaced by hyphens. Never use more or fewer words than this. If two senses in the same response produce the same slug, append -2, -3, etc. to the duplicates.
- `character`: for single-character and compound-word inputs, copy the input exactly. For pinyin inputs, use the matched character(s).
- `pinyin`: full reading with tone marks for the entire `character` value. For pinyin inputs, verify every candidate: strip ALL tone marks from its pinyin and confirm the result is EXACTLY equal to the query. Discard any candidate that fails this check — do not include characters for different base syllables under any circumstances.

EXAMPLES:

Input: '休息' (compound word)
[
  {\"character\": \"休息\", \"pinyin\": \"xiūxi\", \"sense_label\": \"to rest\", \"slug\": \"to-rest\"},
  {\"character\": \"休息\", \"pinyin\": \"xiūxi\", \"sense_label\": \"to take break\", \"slug\": \"to-take\"}
]

Input: 'guo' (pinyin)
[
  {\"character\": \"過\", \"pinyin\": \"guò\", \"sense_label\": \"to pass through\", \"slug\": \"to-pass\"},
  {\"character\": \"過\", \"pinyin\": \"guò\", \"sense_label\": \"experience marker\", \"slug\": \"experience-marker\"},
  {\"character\": \"國\", \"pinyin\": \"guó\", \"sense_label\": \"country\", \"slug\": \"country\"}
]

Input: '過' (single character)
[
  {\"character\": \"過\", \"pinyin\": \"guò\", \"sense_label\": \"to pass through\", \"slug\": \"to-pass\"},
  {\"character\": \"過\", \"pinyin\": \"guò\", \"sense_label\": \"experience marker\", \"slug\": \"experience-marker\"},
  {\"character\": \"過\", \"pinyin\": \"guò\", \"sense_label\": \"to exceed limit\", \"slug\": \"to-exceed\"}
]";

pub async fn fetch_tier1_senses(
    client: &reqwest::Client,
    api_key: &str,
    query: &str,
) -> Result<Vec<SenseCandidate>> {
    let body = serde_json::json!({
        "model": TIER1_MODEL,
        "max_tokens": 1024,
        "system": TIER1_SYSTEM,
        "messages": [
            { "role": "user", "content": query }
        ]
    });

    let res = client
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(anyhow!("Anthropic API error {status}: {text}"));
    }

    let resp: serde_json::Value = res.json().await?;
    let text = resp["content"][0]["text"]
        .as_str()
        .ok_or_else(|| anyhow!("Unexpected API response shape"))?;

    tracing::info!("[Tier 1] raw response: {text}");
    let text = strip_code_fence(text);
    let candidates: Vec<SenseCandidate> = serde_json::from_str(&text)
        .map_err(|e| anyhow!("Failed to parse Tier 1 JSON: {e}\nRaw: {text}"))?;

    tracing::info!("[Tier 1] query: {query:?}");

    for c in &candidates {
        tracing::info!("  {} {} — {}", c.character, c.pinyin, c.sense_label);
    }

    Ok(candidates)
}

fn strip_code_fence(s: &str) -> String {
    let s = s.trim();
    let s = s.strip_prefix("```json").or_else(|| s.strip_prefix("```")).unwrap_or(s);
    let s = s.strip_suffix("```").unwrap_or(s);
    s.trim().to_string()
}
