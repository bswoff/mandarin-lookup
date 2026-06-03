# Mandarin Learning App — Task Breakdown

## How to use this file
At the start of every Claude Code session, read SPEC.md and this file, then
continue from the first unchecked task. Mark tasks complete with [x] as you
finish them. Do not skip ahead — each phase builds on the last.

---

## Phase 1 — Project setup (~1 session)

- [x] Scaffold Rust backend as a Cargo workspace with axum
- [x] Scaffold React frontend with Vite + TypeScript + React Router
- [x] Configure CORS on the backend so the frontend can call it on localhost
- [x] Confirm the connection: a `/api/health` route returns 200 and the frontend can fetch it
- [x] Create the SQLite file and run initial migrations for the `definitions` table (schema in SPEC.md)

---

## Phase 2 — Search (~2 sessions) [deprecated, decided to go with CEDICT impelemntation instead]

- [x] Integrate CC-CEDICT via the `cedict` crate; load `backend/data/cedict.txt` at startup into a `CedictIndex` keyed by normalized pinyin and by character
- [x] Implement `GET /api/search?q={query}`: accepts pinyin (ASCII) or Chinese characters; queries CEDICT directly (no DB cache); returns entries grouped by character+tone, sorted by definition count descending
- [x] Build `SearchBar` component: text input, calls `/api/search` on submit, shows error messages for invalid input or network failures
- [x] Render dropdown grouped by character+tone, with all definitions as inline clickable links separated by ` / `
- [x] Clicking a definition link navigates to `/word/{pinyin_key}/{slug}` where `pinyin_key` is tone-stripped and `slug` is the full definition text slugified
- [x] Test with at least 5 real queries including pinyin (`guo`) and Chinese character (`過`)

---

## Phase 3 — Definition page + definition cache (~2–3 sessions)

- [x] Write the LLM prompt: given a character and definition text from CEDICT, return a single JSON object covering all 7 detail sections (see SPEC.md). Instruct the model to use Traditional Chinese characters throughout and tone marks (not numbers) on all pinyin
- [x] Test the prompt manually against 3–4 words before writing any backend code — iterate until the JSON shape is consistent
- [x] Implement `GET /api/word/{pinyin}/{slug}`: resolve character by querying CEDICT for the pinyin base and matching the slug; check definition cache; on miss call LLM, validate JSON, store, return
- [x] Implement `POST /api/word/{pinyin}/{slug}/regenerate`: delete cache entry, re-call LLM, store and return fresh result
- [x] Build `WordDetail` component rendering all 7 sections:
  - [x] Character + pinyin header
  - [x] Definition
  - [x] Usage summary
  - [x] Grammar pairings table (pairing | notes | example sentence | pinyin | English)
  - [x] Example sentences (character → pinyin → English)
  - [x] Common mistakes
  - [x] Similar words link back to search page pre-filled with the character
- [x] Add a Regenerate button that calls the regenerate endpoint and updates the page in place
- [x] Wire React Router: `/` → search page, `/word/:pinyin/:slug` → detail page

---

## Phase 4 — Polish + MVP sign-off (~1 session)

- [ ] Add loading states for both the search dropdown and the detail page (LLM calls can be slow)
- [ ] Handle errors gracefully: bad LLM JSON, network failures, unknown routes
- [ ] Verify tone marks render correctly throughout (ā á ǎ à not a1 a2 a3 a4)
- [ ] Manual end-to-end test with 10 real words covering different word classes (verb, noun, particle, measure word)
- [ ] Confirm similar word links resolve correctly even when the linked word has no cached definition (should trigger a fresh LLM call)
- [ ] Confirm Regenerate clears the old entry and renders fresh content

---

## TODOs

### Explore word vectorization for more accurate English search queries

Currently English search uses keyword matching with a definition-position ranking heuristic.
Vectorization would allow semantic matching — e.g. searching "slow" surfaces 慢 over 蹇
because "slow" as speed is semantically closer to 慢's meaning than 蹇's primary meaning of
lame/crippled, even though both have "slow" as a valid definition.

- [ ] Evaluate embedding model options: local (e.g. `fastembed`, `candle` in Rust) vs API-based (OpenAI `text-embedding-3-small`)
- [ ] Build a one-time offline script that runs all CEDICT definitions through the chosen model and serializes the resulting vectors to a file (e.g. `backend/data/cedict_embeddings.bin`)
- [ ] At startup, load the vector index from file alongside `cedict.txt` (same pattern as today)
- [ ] On English query, embed the query string and run a nearest-neighbor search against the index; return top-N results ranked by cosine similarity
- [ ] Compare result quality against the current keyword approach on a set of test queries (e.g. "slow", "angry", "to apologize", "drunk")
- [ ] Decide whether to replace keyword search or layer it on top (e.g. vector search for ranking, keyword for filtering)

---

## Out of scope for MVP (do not implement yet)
- Flashcard or spaced repetition mechanics
- Stroke order diagrams
- Audio pronunciation
- Simplified Chinese support
- User accounts or cloud sync
