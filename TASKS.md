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
- [x] Create the SQLite file and run initial migrations for the `senses` and `definitions` tables (schema in SPEC.md)

---

## Phase 2 — Search + Tier 1 cache (~2 sessions)

- [x] Write the Tier 1 LLM prompt: given a pinyin base, return a JSON array of all matching characters across all tones, each sense as its own object with `character`, `pinyin`, `sense_label`, and `slug` (slug = first 1–2 words of sense_label, lowercased, hyphenated)
- [x] Implement `GET /api/search?q={query}`: pinyin only; check Tier 1 cache first; on miss call LLM, store results, return candidates
- [x] Verify slug stability manually: search `guo` twice and confirm identical slugs both times
- [x] Build `SearchBar` component: pinyin text input, debounced, calls `/api/search`
- [x] Render dropdown of candidates showing character + pinyin + sense label
- [x] Selecting a candidate navigates to `/word/:pinyin/:slug`
- [x] Test with at least 5 real pinyin queries including one with many senses (guo) and one with few (xiuxi)

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
### TODOS

## Input validation + error handling

- [ ] Validate search input on the frontend: accept only pinyin (ASCII letters + spaces) or Chinese characters — reject numbers, mixed input, and symbols with an inline error message
- [ ] Surface backend errors to the user: if the search request fails (network error, 5xx), show a visible error message in the UI rather than silently returning no results

## Character search capability

- [ ] Allow users to search by characters as well as pinyin. This has correctness concerns as the LLM may return different responses based on what we search. We should consider removing the LLM call and using `rust-cedit` library instead

## rust-cedit library integration
https://github.com/gkbrk/rust-cedict

- [ ] Explore the usage of this library rather than an LLM call. It seems that different definitions that share a common pinyin and tone are separated by a `/` character. We could use this to add links to each definition, rather than using LLM.
---

## Out of scope for MVP (do not implement yet)
- Flashcard or spaced repetition mechanics
- Stroke order diagrams
- Audio pronunciation
- Simplified Chinese support
- User accounts or cloud sync
