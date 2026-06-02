interface GrammarPairing {
  pairing: string;
  notes: string;
  example: string;
  example_pinyin?: string;
  example_english?: string;
}

interface Props {
  pairings: GrammarPairing[];
}

export default function GrammarTable({ pairings }: Props) {
  return (
    <table style={{
      width: '100%',
      borderCollapse: 'collapse',
      fontSize: 15,
      textAlign: 'left',
    }}>
      <thead>
        <tr style={{ borderBottom: '2px solid var(--border)' }}>
          <th style={{ padding: '8px 12px 8px 0', color: 'var(--text-h)', fontWeight: 600, width: '20%' }}>Pairing</th>
          <th style={{ padding: '8px 12px', color: 'var(--text-h)', fontWeight: 600, width: '30%' }}>Notes</th>
          <th style={{ padding: '8px 0 8px 12px', color: 'var(--text-h)', fontWeight: 600 }}>Example sentence</th>
        </tr>
      </thead>
      <tbody>
        {pairings.map((row, i) => (
          <tr key={i} style={{ borderBottom: '1px solid var(--border)' }}>
            <td style={{ padding: '10px 12px 10px 0', fontWeight: 600, color: 'var(--text-h)', fontSize: 17, verticalAlign: 'top' }}>
              {row.pairing}
            </td>
            <td style={{ padding: '10px 12px', color: 'var(--text)', verticalAlign: 'top' }}>
              {row.notes}
            </td>
            <td style={{ padding: '10px 0 10px 12px', verticalAlign: 'top' }}>
              <div style={{ color: 'var(--text-h)', fontSize: 16 }}>{row.example}</div>
              {row.example_pinyin && (
                <div style={{ color: 'var(--accent)', fontSize: 13, marginTop: 3, letterSpacing: '0.02em' }}>
                  {row.example_pinyin}
                </div>
              )}
              {row.example_english && (
                <div style={{ color: 'var(--text)', fontSize: 13, marginTop: 2, opacity: 0.8 }}>
                  {row.example_english}
                </div>
              )}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
