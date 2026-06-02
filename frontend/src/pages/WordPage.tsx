import { useParams } from 'react-router-dom';

export default function WordPage() {
  const { pinyin, slug } = useParams<{ pinyin: string; slug: string }>();

  return (
    <main style={{ maxWidth: 700, margin: '40px auto', padding: '0 16px' }}>
      <p style={{ color: '#888' }}>
        {pinyin} / {slug} — detail page (Phase 3)
      </p>
    </main>
  );
}
