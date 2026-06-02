import { useState, useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';

interface SearchDefinition {
  sense_label: string;
  slug: string;
}

interface SearchEntry {
  character: string;
  pinyin: string;
  pinyin_key: string;
  definitions: SearchDefinition[];
}

const API = 'http://localhost:3001';

function isValidQuery(q: string): boolean {
  const t = q.trim();
  const isPinyin = t.split('').every(c => (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c === ' ');
  const isChinese = t.split('').every(c => (c >= '一' && c <= '鿿') || (c >= '㐀' && c <= '䶿'));
  return isPinyin || isChinese;
}

export default function SearchBar() {
  const [query, setQuery] = useState('');
  const [entries, setEntries] = useState<SearchEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  useEffect(() => {
    const q = searchParams.get('q');
    if (q) {
      setQuery(q);
      fetchEntries(q);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const fetchEntries = async (q: string) => {
    if (!q.trim()) return;
    if (!isValidQuery(q)) {
      setError('Please enter pinyin (e.g. guo) or Chinese characters (e.g. 過)');
      setEntries([]);
      return;
    }
    setError(null);
    setLoading(true);
    try {
      const res = await fetch(`${API}/api/search?q=${encodeURIComponent(q.trim())}`);
      if (!res.ok) {
        setError(`Search failed (${res.status})`);
        setEntries([]);
        return;
      }
      const data: SearchEntry[] = await res.json();
      setEntries(data);
      if (data.length === 0) setError('No results found');
    } catch {
      setError('Could not reach the server');
      setEntries([]);
    } finally {
      setLoading(false);
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setQuery(e.target.value);
    setEntries([]);
    setError(null);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') fetchEntries(query);
  };

  const handleSelect = (entry: SearchEntry, def: SearchDefinition) => {
    setEntries([]);
    setQuery('');
    setError(null);
    navigate(`/word/${entry.pinyin_key}/${def.slug}`);
  };

  return (
    <div style={{ position: 'relative' }}>
      <div style={{ display: 'flex', gap: 8 }}>
        <input
          value={query}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          placeholder="Search pinyin or character (e.g. guo or 過)..."
          style={{ flex: 1, padding: '10px', fontSize: 18, boxSizing: 'border-box' }}
        />
        <button
          onClick={() => fetchEntries(query)}
          disabled={!query.trim() || loading}
          style={{ padding: '10px 20px', fontSize: 18, cursor: 'pointer' }}
        >
          {loading ? '…' : 'Search'}
        </button>
      </div>
      {error && (
        <p style={{ margin: '6px 0 0', color: '#c00', fontSize: 14 }}>{error}</p>
      )}
      {entries.length > 0 && (
        <ul style={{
          position: 'absolute', top: '100%', left: 0, right: 0,
          background: '#fff', border: '1px solid #ccc', listStyle: 'none',
          margin: 0, padding: 0, zIndex: 10, maxHeight: 400, overflowY: 'auto',
        }}>
          {entries.map((entry) => (
            <li
              key={`${entry.character}-${entry.pinyin}`}
              style={{ padding: '10px 12px', borderBottom: '1px solid #eee', lineHeight: 1.6 }}
            >
              <span style={{ fontWeight: 600 }}>
                {entry.character}&nbsp;{entry.pinyin}
              </span>
              {' — '}
              {entry.definitions.map((def, i) => (
                <span key={def.slug}>
                  <span
                    onClick={() => handleSelect(entry, def)}
                    style={{ cursor: 'pointer', color: '#0066cc', textDecoration: 'underline' }}
                  >
                    {def.sense_label}
                  </span>
                  {i < entry.definitions.length - 1 && ' / '}
                </span>
              ))}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
