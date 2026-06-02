import { useParams } from 'react-router-dom';
import { Link } from 'react-router-dom';

export default function WordPage() {
  const { pinyin, slug } = useParams<{ pinyin: string; slug: string }>();

  return (
    <main style={{ maxWidth: 700, margin: '80px auto', padding: '0 16px' }}>
      <Link to="/" style={{ fontSize: 14, color: '#888' }}>← Back to search</Link>
      <h1 style={{ marginTop: 24, fontSize: 48 }}>{pinyin}</h1>
      <p style={{ color: '#555', fontSize: 20 }}>{slug?.replace(/-/g, ' ')}</p>
      <p style={{ marginTop: 40, color: '#aaa' }}>Definition page coming in Phase 3.</p>
    </main>
  );
}
