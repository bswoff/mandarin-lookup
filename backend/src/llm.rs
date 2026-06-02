use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

pub fn strip_code_fence(s: &str) -> String {
    let s = s.trim();
    let s = s.strip_prefix("```json").or_else(|| s.strip_prefix("```")).unwrap_or(s);
    let s = s.strip_suffix("```").unwrap_or(s);
    s.trim().to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GrammarPairing {
    pub pairing: String,
    pub notes: String,
    pub example: String,
    #[serde(default)]
    pub example_pinyin: String,
    #[serde(default)]
    pub example_english: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExampleSentence {
    pub chinese: String,
    pub pinyin: String,
    pub english: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommonMistake {
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incorrect: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimilarWord {
    pub character: String,
    pub pinyin_key: String,
    pub sense_label: String,
    pub note: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WordDetail {
    pub character: String,
    pub pinyin: String,
    pub definition: String,
    pub usage_summary: String,
    pub grammar_pairings: Vec<GrammarPairing>,
    pub example_sentences: Vec<ExampleSentence>,
    pub common_mistakes: Vec<CommonMistake>,
    pub similar_words: Vec<SimilarWord>,
}

pub async fn generate_word_detail(
    client: &reqwest::Client,
    api_key: &str,
    character: &str,
    definition_text: &str,
) -> Result<WordDetail> {
    let prompt = build_prompt(character, definition_text);

    let body = serde_json::json!({
        "model": "claude-sonnet-4-6",
        "max_tokens": 4096,
        "messages": [{ "role": "user", "content": prompt }]
    });

    let resp = client
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("Anthropic API error {}: {}", status, text));
    }

    let resp_json: serde_json::Value = resp.json().await?;
    let raw = resp_json["content"][0]["text"]
        .as_str()
        .ok_or_else(|| anyhow!("Unexpected API response shape: {:?}", resp_json))?;

    tracing::debug!(raw = %raw, "raw LLM response");

    let cleaned = strip_code_fence(raw);
    let detail: WordDetail = serde_json::from_str(&cleaned)
        .map_err(|e| anyhow!("Failed to parse LLM JSON: {}\n\nRaw:\n{}", e, cleaned))?;

    Ok(detail)
}

fn build_prompt(character: &str, definition_text: &str) -> String {
    format!(
        r#"You are a Mandarin Chinese language expert. Given a Chinese character/word and its CC-CEDICT definition, produce a detailed learning resource.

Character: {character}
CEDICT definition: {definition_text}

Return ONLY a raw JSON object (no markdown fences, no explanation) with exactly this structure:

{{
  "character": "<Traditional Chinese characters>",
  "pinyin": "<pinyin with tone marks, e.g. guò — never tone numbers like guo4>",
  "definition": "<the specific sense from the CEDICT definition, stated clearly>",
  "usage_summary": "<1-2 paragraphs on what this word conveys in this sense, its register (formal/colloquial/written/slang), and common contexts>",
  "grammar_pairings": [
    {{
      "pairing": "<grammatical construction using Traditional Chinese>",
      "notes": "<when and how to use this construction>",
      "example": "<one example sentence in Traditional Chinese characters>",
      "example_pinyin": "<pinyin for the example sentence with tone marks>",
      "example_english": "<English translation of the example sentence>"
    }}
  ],
  "example_sentences": [
    {{
      "chinese": "<Traditional Chinese characters>",
      "pinyin": "<full sentence pinyin with tone marks on every syllable>",
      "english": "<English translation>"
    }}
  ],
  "common_mistakes": [
    {{
      "description": "<description of the error pattern>",
      "incorrect": "<incorrect usage in Traditional Chinese — omit this field if no concrete example is helpful>",
      "correction": "<correct form in Traditional Chinese — omit this field if no concrete example is helpful>"
    }}
  ],
  "similar_words": [
    {{
      "character": "<Traditional Chinese characters>",
      "pinyin_key": "<ASCII pinyin, lowercase, no spaces, no tone marks — e.g. guo for 過, xiuxi for 休息>",
      "sense_label": "<short English definition of the specific sense being linked, e.g. 'to surpass' or 'to pass by'>",
      "note": "<one line distinguishing this word from {character} in this specific sense>"
    }}
  ]
}}

Constraints:
- Use Traditional Chinese characters throughout, never Simplified
- All pinyin must use tone mark characters (ā á ǎ à ē é ě è etc.), never tone numbers
- grammar_pairings: 2–4 entries covering the main grammatical patterns for this sense
- example_sentences: 3–5 sentences covering distinct usages of this specific sense
- common_mistakes: 2–4 typical learner errors specific to this sense
- similar_words: 3–5 semantically related words relevant to THIS specific sense
- pinyin_key must be lowercase ASCII letters only, no spaces, no tone marks"#,
        character = character,
        definition_text = definition_text,
    )
}
