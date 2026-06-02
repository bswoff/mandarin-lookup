import { useEffect, useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import WordDetail, { type WordDetailData } from '../components/WordDetail';

const API = 'http://localhost:3001';

export default function WordPage() {
  const { pinyin, slug } = useParams<{ pinyin: string; slug: string }>();
  const [data, setData] = useState<WordDetailData | null>(null);
  const [loading, setLoading] = useState(true);
  const [uncached, setUncached] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [generating, setGenerating] = useState(false);

  useEffect(() => {
    if (!pinyin || !slug) return;
    const controller = new AbortController();

    setLoading(true);
    setError(null);
    setUncached(false);

    fetch(`${API}/api/word/${pinyin}/${slug}`, { signal: controller.signal })
      .then(res => {
        if (res.status === 404) throw new Error('Word not found — the dictionary entry may not exist.');
        if (!res.ok) throw new Error(`Failed to load definition (${res.status})`);
        return res.json();
      })
      .then(json => {
        if (json.cached === false) {
          setUncached(true);
        } else {
          setData(json as WordDetailData);
        }
      })
      .catch(err => {
        if (err.name === 'AbortError') return;
        setError(err.message ?? 'Could not reach the server');
      })
      .finally(() => setLoading(false));

    return () => controller.abort();
  }, [pinyin, slug]);

  const handleGenerate = async () => {
    if (!pinyin || !slug || generating) return;
    setGenerating(true);
    setError(null);
    try {
      const res = await fetch(`${API}/api/word/${pinyin}/${slug}/regenerate`, { method: 'POST' });
      if (!res.ok) {
        setError(`Generation failed (${res.status})`);
        return;
      }
      const json: WordDetailData = await res.json();
      setData(json);
      setUncached(false);
    } catch {
      setError('Could not reach the server');
    } finally {
      setGenerating(false);
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

      {!loading && uncached && !data && (
        <div style={{ marginTop: 48 }}>
          <code style={{ fontSize: 13, color: 'var(--text)', opacity: 0.5 }}>
            /word/{pinyin}/{slug}
          </code>
          <p style={{ marginTop: 20, fontSize: 18, color: 'var(--text-h)' }}>
            Definition not generated
          </p>
          <button
            onClick={handleGenerate}
            disabled={generating}
            style={{
              marginTop: 16,
              padding: '10px 24px',
              fontSize: 15,
              cursor: generating ? 'not-allowed' : 'pointer',
              background: 'var(--accent)',
              color: '#fff',
              border: 'none',
              borderRadius: 6,
              opacity: generating ? 0.6 : 1,
            }}
          >
            {generating ? 'Generating…' : 'Generate'}
          </button>
        </div>
      )}

      {!loading && !error && data && (
        <div style={{ marginTop: 32 }}>
          <WordDetail data={data} onRegenerate={handleGenerate} regenerating={generating} />
        </div>
      )}
    </main>
  );
}
