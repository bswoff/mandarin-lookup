import { useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';

interface Candidate {
  character: string;
  pinyin: string;
  sense_label: string;
  slug: string;
}

const API = 'http://localhost:3001';

function useDebounce<T extends unknown[]>(fn: (...args: T) => void, delay: number) {
  let timer: ReturnType<typeof setTimeout>;
  return useCallback((...args: T) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), delay);
  }, [fn, delay]);
}

export default function SearchBar() {
  const [query, setQuery] = useState('');
  const [candidates, setCandidates] = useState<Candidate[]>([]);
  const navigate = useNavigate();

  const fetchCandidates = async (q: string) => {
    if (!q.trim()) { setCandidates([]); return; }
    try {
      const res = await fetch(`${API}/api/search?q=${encodeURIComponent(q)}`);
      const data = await res.json();
      setCandidates(data);
    } catch {
      setCandidates([]);
    }
  };

  const debouncedFetch = useDebounce(fetchCandidates, 300);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value;
    setQuery(val);
    debouncedFetch(val);
  };

  const handleSelect = (c: Candidate) => {
    setCandidates([]);
    setQuery('');
    navigate(`/word/${c.pinyin}/${c.slug}`);
  };

  return (
    <div style={{ position: 'relative' }}>
      <input
        value={query}
        onChange={handleChange}
        placeholder="Search pinyin or character..."
        style={{ width: '100%', padding: '10px', fontSize: 18, boxSizing: 'border-box' }}
      />
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
              <strong>{c.character}</strong> &nbsp;{c.pinyin}&nbsp;
              <span style={{ color: '#555' }}>— {c.sense_label}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
