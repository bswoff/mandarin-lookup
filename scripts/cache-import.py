#!/usr/bin/env python3
"""
Write a manually-generated word detail JSON into the definition cache.

Usage (from repo root):
  python3 scripts/cache-import.py <slug> <json-file>

Example:
  python3 scripts/cache-import.py to-introduce-sb-to-sb response.json

The slug comes from the word page URL: /word/{pinyin}/{slug}
The JSON file is the raw output from Claude — save it to a .json file first.
"""

import sys
import json
import sqlite3
from pathlib import Path

if len(sys.argv) != 3:
    print(__doc__)
    sys.exit(1)

slug = sys.argv[1]
json_path = Path(sys.argv[2])

if not json_path.exists():
    print(f"Error: file not found: {json_path}")
    sys.exit(1)

try:
    with open(json_path, encoding="utf-8") as f:
        data = json.load(f)
except json.JSONDecodeError as e:
    print(f"Error: invalid JSON — {e}")
    sys.exit(1)

character = data.get("character")
if not character:
    print("Error: JSON is missing the 'character' field")
    sys.exit(1)

db_path = Path(__file__).parent.parent / "backend" / "mandarin.db"
if not db_path.exists():
    print(f"Error: database not found at {db_path} — is the backend directory correct?")
    sys.exit(1)

conn = sqlite3.connect(db_path)
conn.execute(
    "INSERT OR REPLACE INTO definitions (character, slug, data) VALUES (?, ?, ?)",
    (character, slug, json.dumps(data, ensure_ascii=False)),
)
conn.commit()
conn.close()

print(f"✓  Cached  {character}  /  {slug}")
