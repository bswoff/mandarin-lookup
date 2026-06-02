// Tier 2 LLM integration (definition page) — implemented in Phase 3.

#[allow(dead_code)]
pub const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

#[allow(dead_code)]
pub fn strip_code_fence(s: &str) -> String {
    let s = s.trim();
    let s = s.strip_prefix("```json").or_else(|| s.strip_prefix("```")).unwrap_or(s);
    let s = s.strip_suffix("```").unwrap_or(s);
    s.trim().to_string()
}
