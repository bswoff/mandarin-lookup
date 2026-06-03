import { Link } from 'react-router-dom';
import GrammarTable from './GrammarTable';

interface GrammarPairing {
  pairing: string;
  notes: string;
  example: string;
}

interface ExampleSentence {
  chinese: string;
  pinyin: string;
  english: string;
}

interface CommonMistake {
  description: string;
  incorrect?: string;
  correction?: string;
}

interface SimilarWord {
  character: string;
  pinyin_key: string;
  sense_label: string;
  note: string;
}

export interface WordDetailData {
  character: string;
  pinyin: string;
  definition: string;
  usage_summary: string;
  grammar_pairings: GrammarPairing[];
  example_sentences: ExampleSentence[];
  common_mistakes: CommonMistake[];
  similar_words: SimilarWord[];
}


const sectionHeadingStyle: React.CSSProperties = {
  fontSize: 13,
  fontWeight: 600,
  letterSpacing: '0.08em',
  textTransform: 'uppercase',
  color: 'var(--accent)',
  margin: '32px 0 10px',
};

interface Props {
  data: WordDetailData;
  onRegenerate: () => void;
  regenerating: boolean;
  onModify: () => void;
}

export default function WordDetail({ data, onRegenerate, regenerating, onModify }: Props) {
  return (
    <div>
      {/* Header: character, pinyin, regenerate */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', flexWrap: 'wrap', gap: 12 }}>
        <div>
          <div style={{ fontSize: 72, lineHeight: 1, color: 'var(--text-h)', fontWeight: 400 }}>
            {data.character}
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginTop: 6 }}>
            <div style={{ fontSize: 22, color: 'var(--accent)', letterSpacing: '0.04em' }}>
              {data.pinyin}
            </div>
            <a
              href={`https://youglish.com/pronounce/${encodeURIComponent(data.character)}/chinese`}
              target="_blank"
              rel="noopener noreferrer"
              title="Hear pronunciation on YouGlish"
              style={{
                fontSize: 20,
                opacity: 0.7,
                lineHeight: 1,
                color: 'var(--text)',
                textDecoration: 'none',
              }}
            >
              🔊
            </a>
          </div>
        </div>
        <div style={{ display: 'flex', gap: 8, marginTop: 8 }}>
          <button
            onClick={onModify}
            style={{
              padding: '8px 16px',
              fontSize: 14,
              cursor: 'pointer',
              background: 'transparent',
              border: '1px solid var(--border)',
              borderRadius: 6,
              color: 'var(--text)',
            }}
          >
            Modify JSON
          </button>
          <button
            onClick={onRegenerate}
            disabled={regenerating}
            style={{
              padding: '8px 16px',
              fontSize: 14,
              cursor: regenerating ? 'not-allowed' : 'pointer',
              background: 'transparent',
              border: '1px solid var(--border)',
              borderRadius: 6,
              color: 'var(--text)',
              opacity: regenerating ? 0.5 : 1,
            }}
          >
            {regenerating ? 'Regenerating…' : 'Regenerate'}
          </button>
        </div>
      </div>

      {/* Section 2: Definition */}
      <p style={sectionHeadingStyle}>Definition</p>
      <p style={{ fontSize: 18, color: 'var(--text-h)', lineHeight: 1.5 }}>{data.definition}</p>

      {/* Section 3: Usage Summary */}
      <p style={sectionHeadingStyle}>Usage</p>
      <p style={{ lineHeight: 1.7, color: 'var(--text)', whiteSpace: 'pre-wrap' }}>{data.usage_summary}</p>

      {/* Section 4: Grammar Pairings */}
      <p style={sectionHeadingStyle}>Grammar pairings</p>
      <GrammarTable pairings={data.grammar_pairings} />

      {/* Section 5: Example Sentences */}
      <p style={sectionHeadingStyle}>Example sentences</p>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
        {data.example_sentences.map((s, i) => (
          <div key={i} style={{ borderLeft: '3px solid var(--accent-border)', paddingLeft: 14 }}>
            <div style={{ fontSize: 18, color: 'var(--text-h)', lineHeight: 1.5 }}>{s.chinese}</div>
            <div style={{ fontSize: 14, color: 'var(--accent)', marginTop: 2, letterSpacing: '0.02em' }}>{s.pinyin}</div>
            <div style={{ fontSize: 15, color: 'var(--text)', marginTop: 4 }}>{s.english}</div>
          </div>
        ))}
      </div>

      {/* Section 6: Common Mistakes */}
      <p style={sectionHeadingStyle}>Common mistakes</p>
      <ul style={{ paddingLeft: 20, margin: 0, display: 'flex', flexDirection: 'column', gap: 14 }}>
        {data.common_mistakes.map((m, i) => (
          <li key={i} style={{ color: 'var(--text)', lineHeight: 1.6 }}>
            <span>{m.description}</span>
            {(m.incorrect || m.correction) && (
              <div style={{ marginTop: 6, display: 'flex', gap: 16, flexWrap: 'wrap' }}>
                {m.incorrect && (
                  <span style={{ fontSize: 14, background: 'rgba(200,0,0,0.07)', padding: '2px 8px', borderRadius: 4, color: '#c00' }}>
                    ✗ {m.incorrect}
                  </span>
                )}
                {m.correction && (
                  <span style={{ fontSize: 14, background: 'rgba(0,150,0,0.07)', padding: '2px 8px', borderRadius: 4, color: '#080' }}>
                    ✓ {m.correction}
                  </span>
                )}
              </div>
            )}
          </li>
        ))}
      </ul>

      {/* Section 7: Similar Words */}
      <p style={sectionHeadingStyle}>Similar words</p>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
        {data.similar_words.map((w, i) => (
            <div key={i} style={{ display: 'flex', gap: 14, alignItems: 'baseline', flexWrap: 'wrap' }}>
              <Link
                to={`/?q=${encodeURIComponent(w.character)}`}
                style={{ fontSize: 22, color: 'var(--accent)', textDecoration: 'none', fontWeight: 500 }}
              >
                {w.character}
              </Link>
              <span style={{ fontSize: 14, color: 'var(--text)', flex: 1 }}>
                <em style={{ color: 'var(--text-h)' }}>{w.sense_label}</em>
                {' — '}
                {w.note}
              </span>
            </div>
        ))}
      </div>
    </div>
  );
}
