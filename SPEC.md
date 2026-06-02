# Mandarin Learning App — Project Spec

## Overview

A personal browser-based Mandarin learning tool. Users look up Chinese words, get
rich LLM-generated detail about each word sense, and build a cached personal dictionary
over time.

**Stack:** React (frontend) + Rust (backend, HTTP API) + SQLite (local cache)  
**Runtime:** localhost only

---

## User Flow

1. User types a pinyin query (e.g. `guo`) or a Chinese character/word (e.g. `過`)
2. A dropdown appears with matching entries from CC-CEDICT, grouped by character+tone, showing all definitions as inline links (e.g. "to cross / to go over / to pass (time) / …")
3. User clicks a definition link
4. App navigates to a dedicated word detail page at a clean URL (e.g. `/word/guo/to-cross`)

The detail page is preferred over a popup or inline expansion because:
- The content is substantial enough that a popup would feel cramped
- A clean URL enables bookmarking and linking
- "Similar words" navigation feels natural as page-to-page links

Pages are **definition-level, not word-level.** A word like 過 has many distinct
senses (to pass, experience marker, to celebrate, excessively) that each have their
own grammar patterns, common mistakes, and similar words. Each sense gets its own page.

---

## URL Schema

```
/word/{pinyin}/{slug}
```

- `pinyin` — the pinyin of the character without tone marks (e.g. `guo`)
- `slug` — derived deterministically from the full CEDICT definition text: replace
  any run of non-alphanumeric characters with a single hyphen, lowercase, trim
  trailing hyphens. Examples:
  - "to cross" → `to-cross`
  - "to pass (time)" → `to-pass-time`
  - "to celebrate (a holiday)" → `to-celebrate-a-holiday`
  - "excessively" → `excessively`

Slugs are generated at search time from the CEDICT data and are stable as long as
the dictionary file does not change.

---

## Search

Search queries the CC-CEDICT dictionary bundled at `backend/data/cedict.txt`.
Results are returned directly — no database caching. Supported query types:

- **Pinyin** (ASCII letters, e.g. `guo`): returns all characters across all tones whose
  base syllable (tone marks stripped, spaces removed) exactly matches the query.
- **Chinese character/word** (e.g. `過` or `休息`): returns all CEDICT entries for that
  exact character string.

Results are grouped by character+tone (one row per CEDICT entry) and sorted descending
by number of definitions. Each row shows all definitions as inline links.

## Definition Cache

**Populated:** when a user first clicks through to a definition page  
**Cache key:** `(character, slug)`  
**Stores:** the full structured JSON for that word sense

On a miss, the backend calls the LLM with the character and definition text as context,
parses and validates the JSON response, stores it, and returns it.
On a hit, it returns the cached JSON directly.

### Cache Invalidation
Each definition page has a "Regenerate" button that:
1. Deletes the cache entry for `(character, slug)`
2. Re-calls the LLM
3. Stores and returns the fresh result

---

## Word Detail Page

Each definition page displays the following sections, in order:

### 1. Character + Pinyin
- Full traditional character(s)
- Pinyin with tone marks (not numbers)

### 2. Definition
- The definition for this specific sense, clearly stated

### 3. Usage Summary
- A short paragraph on what the word conveys in this sense, its register
  (formal / colloquial / written / slang), and common contexts

### 4. Grammar Pairings
Displayed as a table:

| Pairing | Notes | Example sentence |
|---------|-------|-----------------|
| 介紹給   | Most common construction | 我來介紹給你認識。 |
| 介紹 (standalone) | Used in formal introductions | 請讓我介紹一下。 |

Each pairing gets exactly one example sentence so the grammar point is immediately concrete.

### 5. Example Sentences
- 3–5 sentences, chosen to cover the distinct ways this sense can be used
- Each sentence: Chinese characters → pinyin (with tones on every word) → English translation
- Number of sentences scales with how many distinct usages exist

### 6. Common Mistakes
- A short list of errors learners commonly make with this word sense
- Include incorrect example + correction where helpful

### 7. Similar Words
- 3–5 related words for this specific sense
- Each displayed as a link that navigates to that word's own detail page
- Include a one-line note distinguishing it from the current word

---

## Backend

### API Endpoints

**Search**
```
GET /api/search?q={query}
```
Queries CC-CEDICT for all matching entries. Returns a list grouped by character+tone:
`[{ character, pinyin, pinyin_key, definitions: [{ sense_label, slug }] }]`.

**Word detail**
```
GET /api/word/{pinyin}/{slug}
```
Resolves the character by querying CC-CEDICT for the pinyin base and matching the slug.
Then checks the definition cache. On miss, calls the LLM with character + definition text
as context, stores result, returns full structured JSON.

**Regenerate**
```
POST /api/word/{pinyin}/{slug}/regenerate
```
Resolves the character the same way, deletes the Tier 2 cache entry, re-generates it,
and returns the fresh JSON.

### SQLite Schema

```sql
-- Definition cache: full detail JSON per sense
CREATE TABLE definitions (
  character   TEXT NOT NULL,
  slug        TEXT NOT NULL,
  data        TEXT NOT NULL,  -- JSON blob
  created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (character, slug)
);
```

### LLM Integration
- Use the Anthropic API (`claude-sonnet-4-20250514`)
- Prompt: given a character and definition text from CEDICT, return a JSON object covering
  all seven detail sections
- Parse and validate JSON before storing or returning
- All output should use Traditional Chinese characters throughout

---

## Frontend

### Pages / Routes
- `/` — Search home
- `/word/:pinyin/:slug` — Word detail page (pinyin matches backend API; character is resolved server-side from the senses table)

### Search Component
- Single text input (pinyin or Chinese characters)
- Dropdown appears on search, grouped by character+tone
- Each row shows: character + pinyin — def1 / def2 / def3 (each definition is a link)
- Clicking a definition link navigates to its detail page

### Word Detail Component
- Renders all seven sections defined above
- Grammar pairings rendered as a table
- Similar words rendered as router links
- "Regenerate" button in the header area
- Pinyin uses proper tone mark characters (ā á ǎ à), not numbers

---

## Project Structure (suggested)

```
/
├── SPEC.md               ← this file
├── frontend/             ← React app (Vite recommended)
│   └── src/
│       ├── App.tsx
│       ├── pages/
│       │   ├── SearchPage.tsx
│       │   └── WordPage.tsx
│       └── components/
│           ├── SearchBar.tsx
│           ├── WordDetail.tsx
│           └── GrammarTable.tsx
└── backend/              ← Rust (axum or actix-web recommended)
    └── src/
        ├── main.rs
        ├── routes/
        │   ├── search.rs
        │   └── word.rs
        ├── llm.rs        ← Anthropic API client
        └── db.rs         ← SQLite cache (rusqlite or sqlx)
```

---

## Out of Scope (for now)
- Authentication
- Flashcard / spaced repetition mechanics
- Stroke order diagrams
- Audio pronunciation
- User accounts or cloud sync
- Simplified Chinese support
