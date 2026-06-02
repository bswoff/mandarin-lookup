# mandarin-lookup

A localhost browser for looking up definitions and usage patterns for Chinese characters and words, powered by Claude.

## Running locally

**Backend** (terminal 1, from `backend/`):
```bash
source "$HOME/.cargo/env"
cargo run
```

**Frontend** (terminal 2, from `frontend/`):
```bash
npm run dev
```

Then open http://localhost:5173. Logs are written to `backend/mandarin.log`.

## Environment

Copy `.env.example` to `backend/.env` and add your Anthropic API key:
```
ANTHROPIC_API_KEY=sk-ant-...
```

## Known limitations

**Pinyin-only search**
Search accepts only pinyin (ASCII letters). Searching by Chinese character (e.g. `過`) is not currently supported.

**Pinyin completeness is locked at first search**
The first time a pinyin syllable (e.g. `guo`) is searched, the LLM returns all matching characters and that result is cached permanently. If the first call returned an incomplete set of characters, subsequent searches for the same syllable will return the same incomplete set.

**Clearing the cache**
The `senses` and `definitions` tables are independent and can be cleared separately, but clearing `senses` without `definitions` may leave orphaned definition rows. Clear both together to reset fully:
```bash
sqlite3 backend/mandarin.db "DELETE FROM senses; DELETE FROM definitions;"
```
