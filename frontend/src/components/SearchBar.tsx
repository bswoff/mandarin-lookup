import { useState } from 'react';
import { useNavigate } from 'react-router-dom';

interface Candidate {
  character: string;
  pinyin: string;
  sense_label: string;
  slug: string;
}

const API = 'http://localhost:3001';

export default function SearchBar() {
  const [query, setQuery] = useState('');
  const [candidates, setCandidates] = useState<Candidate[]>([]);
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  const isPinyin = (q: string) => /^[a-zA-Z\s]+$/.test(q.trim());

  const fetchCandidates = async (q: string) => {
    if (!q.trim()) return;
    if (!isPinyin(q)) return;
    setLoading(true);
    try {
      const res = await fetch(`${API}/api/search?q=${encodeURIComponent(q.trim())}`);
      const data = await res.json();
      setCandidates(data);
    } catch {
      setCandidates([]);
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') fetchCandidates(query);
  };

  const handleSelect = (c: Candidate) => {
    setCandidates([]);
    setQuery('');
    navigate(`/word/${c.pinyin}/${c.slug}`);
  };

  return (
    <div style={{ position: 'relative' }}>
      <div style={{ display: 'flex', gap: 8 }}>
        <input
          value={query}
          onChange={(e) => { setQuery(e.target.value); setCandidates([]); }}
          onKeyDown={handleKeyDown}
          placeholder="Search pinyin (e.g. guo)..."
          style={{ flex: 1, padding: '10px', fontSize: 18, boxSizing: 'border-box' }}
        />
        <button
          onClick={() => fetchCandidates(query)}
          disabled={!query.trim() || loading}
          style={{ padding: '10px 20px', fontSize: 18, cursor: 'pointer' }}
        >
          {loading ? '…' : 'Search'}
        </button>
      </div>
      {candidates.length > 0 && (
        <ul style={{
          position: 'absolute', top: '100%', left: 0, right: 0,
          background: '#fff', border: '1px solid #ccc', listStyle: 'none',
          margin: 0, padding: 0, zIndex: 10,
        }}>
          {candidates.map((c) => (
            <li
              key={`${c.character}-${c.slug}`}
              onClick={() => handleSelect(c)}
              style={{ padding: '10px 12px', cursor: 'pointer', borderBottom: '1px solid #eee' }}
            >
              <span style={{ color: '#555' }}>{c.character}&nbsp;{c.pinyin}</span>&nbsp;— <strong>{c.sense_label}</strong>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
