import { useEffect, useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import WordDetail, { type WordDetailData } from '../components/WordDetail';

const API = 'http://localhost:3001';

export default function WordPage() {
  const { pinyin, slug } = useParams<{ pinyin: string; slug: string }>();
  const [data, setData] = useState<WordDetailData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [regenerating, setRegenerating] = useState(false);

  useEffect(() => {
    if (!pinyin || !slug) return;
    const controller = new AbortController();

    setLoading(true);
    setError(null);

    fetch(`${API}/api/word/${pinyin}/${slug}`, { signal: controller.signal })
      .then(res => {
        if (res.status === 404) throw new Error('Word not found — the dictionary entry may not exist.');
        if (!res.ok) throw new Error(`Failed to load definition (${res.status})`);
        return res.json() as Promise<WordDetailData>;
      })
      .then(json => setData(json))
      .catch(err => {
        if (err.name === 'AbortError') return;
        setError(err.message ?? 'Could not reach the server');
      })
      .finally(() => setLoading(false));

    return () => controller.abort();
  }, [pinyin, slug]);

  const handleRegenerate = async () => {
    if (!pinyin || !slug || regenerating) return;
    setRegenerating(true);
    setError(null);
    try {
      const res = await fetch(`${API}/api/word/${pinyin}/${slug}/regenerate`, { method: 'POST' });
      if (!res.ok) {
        setError(`Regenerate failed (${res.status})`);
        return;
      }
      const json: WordDetailData = await res.json();
      setData(json);
    } catch {
      setError('Could not reach the server');
    } finally {
      setRegenerating(false);
    }
  };

  return (
    <main style={{ maxWidth: 720, margin: '0 auto', padding: '48px 24px 80px' }}>
      <Link
        to="/"
        style={{ fontSize: 14, color: 'var(--text)', textDecoration: 'none', opacity: 0.7 }}
      >
        ← Back to search
      </Link>

      {loading && (
        <div style={{ marginTop: 48, color: 'var(--text)', fontSize: 16, opacity: 0.6 }}>
          Loading…
        </div>
      )}

      {error && (
        <div style={{ marginTop: 48, color: '#c00', fontSize: 15 }}>{error}</div>
      )}

      {!loading && !error && data && (
        <div style={{ marginTop: 32 }}>
          <WordDetail data={data} onRegenerate={handleRegenerate} regenerating={regenerating} />
        </div>
      )}
    </main>
  );
}
