import { useState } from 'react';

interface Props {
  title: string;
  initialJson: string;
  onSave: (json: string) => Promise<void>;
  onClose: () => void;
}

export default function JsonEditorModal({ title, initialJson, onSave, onClose }: Props) {
  const [json, setJson] = useState(initialJson);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const handleSave = async () => {
    try {
      JSON.parse(json);
    } catch (e) {
      setError(`Invalid JSON: ${(e as Error).message}`);
      return;
    }
    setSaving(true);
    setError(null);
    try {
      await onSave(json);
    } catch (e) {
      setError((e as Error).message ?? 'Save failed');
    } finally {
      setSaving(false);
    }
  };

  return (
    <div
      onClick={onClose}
      style={{
        position: 'fixed',
        inset: 0,
        background: 'rgba(0,0,0,0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
      }}
    >
      <div
        onClick={e => e.stopPropagation()}
        style={{
          background: 'var(--bg)',
          border: '1px solid var(--border)',
          borderRadius: 8,
          padding: 24,
          width: '90vw',
          maxWidth: 760,
          maxHeight: '90vh',
          display: 'flex',
          flexDirection: 'column',
          gap: 12,
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: 16, fontWeight: 600, color: 'var(--text-h)' }}>{title}</span>
          <button
            onClick={onClose}
            style={{
              background: 'transparent',
              border: 'none',
              cursor: 'pointer',
              fontSize: 18,
              color: 'var(--text)',
              opacity: 0.6,
              lineHeight: 1,
              padding: 4,
            }}
          >
            ✕
          </button>
        </div>
        <textarea
          value={json}
          onChange={e => setJson(e.target.value)}
          style={{
            minHeight: 400,
            fontFamily: 'var(--mono)',
            fontSize: 13,
            padding: 12,
            background: 'var(--code-bg)',
            color: 'var(--text-h)',
            border: '1px solid var(--border)',
            borderRadius: 4,
            resize: 'vertical',
            lineHeight: 1.5,
          }}
          spellCheck={false}
        />
        {error && <div style={{ color: '#c00', fontSize: 13 }}>{error}</div>}
        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8 }}>
          <button
            onClick={onClose}
            disabled={saving}
            style={{
              padding: '8px 16px',
              fontSize: 14,
              cursor: saving ? 'not-allowed' : 'pointer',
              background: 'transparent',
              border: '1px solid var(--border)',
              borderRadius: 6,
              color: 'var(--text)',
            }}
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            style={{
              padding: '8px 16px',
              fontSize: 14,
              cursor: saving ? 'not-allowed' : 'pointer',
              background: 'var(--accent)',
              color: '#fff',
              border: 'none',
              borderRadius: 6,
              opacity: saving ? 0.6 : 1,
            }}
          >
            {saving ? 'Saving…' : 'Save'}
          </button>
        </div>
      </div>
    </div>
  );
}
