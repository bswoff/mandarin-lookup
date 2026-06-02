# Mandarin Learning App — Task Breakdown

## How to use this file
At the start of every Claude Code session, read SPEC.md and this file, then
continue from the first unchecked task. Mark tasks complete with [x] as you
finish them. Do not skip ahead — each phase builds on the last.

---

## Phase 1 — Project setup (~1 session)

- [ ] Scaffold Rust backend as a Cargo workspace with axum
- [ ] Scaffold React frontend with Vite + TypeScript + React Router
- [ ] Configure CORS on the backend so the frontend can call it on localhost
- [ ] Confirm the connection: a `/api/health` route returns 200 and the frontend can fetch it
- [ ] Create the SQLite file and run initial migrations for the `senses` and `definitions` tables (schema in SPEC.md)

---

## Phase 2 — Search + Tier 1 cache (~2 sessions)

- [ ] Write the Tier 1 LLM prompt: given a character or pinyin, return a JSON array of all distinct senses, each with `character`, `pinyin`, `sense_label`, and `slug` (slug = first 1–2 words of sense_label, lowercased, hyphenated)
- [ ] Implement `GET /api/search?q={query}`: check Tier 1 cache first; on miss call LLM, store results, return candidates
- [ ] Verify slug stability manually: search 過 twice and confirm identical slugs both times
- [ ] Build `SearchBar` component: text input, debounced, calls `/api/search`
- [ ] Render dropdown of candidates showing character + pinyin + sense label
- [ ] Selecting a candidate navigates to `/word/:pinyin/:slug`
- [ ] Test with at least 5 real words including one with many senses (過) and one with few (休息)

---

## Phase 3 — Definition page + Tier 2 cache (~2–3 sessions)

- [ ] Write the Tier 2 LLM prompt: given a character and sense_label, return a single JSON object covering all 7 detail sections (see SPEC.md for exact structure). Instruct the model to use Traditional Chinese characters throughout and tone marks (not numbers) on all pinyin
- [ ] Test the Tier 2 prompt manually against 3–4 words before writing any backend code — iterate until the JSON shape is consistent
- [ ] Implement `GET /api/word/:character/:slug`: check Tier 2 cache; on miss call LLM, validate JSON, store, return
- [ ] Implement `POST /api/word/:character/:slug/regenerate`: delete Tier 2 entry, re-call LLM, store and return fresh result
- [ ] Build `WordDetail` component rendering all 7 sections:
  - [ ] Character + pinyin header
  - [ ] Definition
  - [ ] Usage summary
  - [ ] Grammar pairings table (pairing | notes | example sentence)
  - [ ] Example sentences (character → pinyin → English)
  - [ ] Common mistakes
  - [ ] Similar words as router links to their own detail pages
- [ ] Add a Regenerate button that calls the regenerate endpoint and updates the page in place
- [ ] Wire React Router: `/` → search page, `/word/:pinyin/:slug` → detail page

---

## Phase 4 — Polish + MVP sign-off (~1 session)

- [ ] Add loading states for both the search dropdown and the detail page (LLM calls can be slow)
- [ ] Handle errors gracefully: bad LLM JSON, network failures, unknown routes
- [ ] Verify tone marks render correctly throughout (ā á ǎ à not a1 a2 a3 a4)
- [ ] Manual end-to-end test with 10 real words covering different word classes (verb, noun, particle, measure word)
- [ ] Confirm similar word links resolve correctly even when the linked word is not yet cached (should trigger a fresh Tier 1 + Tier 2 call)
- [ ] Confirm Regenerate clears the old entry and renders fresh content

---

## Out of scope for MVP (do not implement yet)
- Flashcard or spaced repetition mechanics
- Stroke order diagrams
- Audio pronunciation
- Simplified Chinese support
- User accounts or cloud sync
