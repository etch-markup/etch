import { EtchEditor } from '@etch-markup/etch-react';
import wasmUrl from 'etch-wasm/etch_wasm_bg.wasm?url';

const INITIAL_SOURCE = `# Etch Web Editor

Write on the left. Preview on the right.

Inline math works: :math[\\frac{1}{1 + x}]

::note[Tip]
Theme switching is handled in the browser using the built-in Etch themes.
::

## Checklist

- Live preview
- Parse diagnostics
- Browser-hosted WASM
`;

export function App() {
  return (
    <main
      style={{
        minHeight: '100vh',
        padding: 'clamp(1rem, 3vw, 2rem)',
        background:
          'radial-gradient(circle at top left, rgba(186, 230, 253, 0.65), transparent 26%), radial-gradient(circle at top right, rgba(251, 191, 36, 0.22), transparent 24%), linear-gradient(180deg, #f8fafc 0%, #eef2ff 100%)',
        color: '#0f172a',
      }}
    >
      <div
        style={{
          width: 'min(100%, 92rem)',
          margin: '0 auto',
          display: 'grid',
          gap: '1.5rem',
        }}
      >
        <header
          style={{
            display: 'grid',
            gap: '0.5rem',
            padding: '1.5rem',
            borderRadius: '1.5rem',
            background: 'rgba(255, 255, 255, 0.78)',
            border: '1px solid rgba(148, 163, 184, 0.2)',
            boxShadow: '0 24px 60px rgba(15, 23, 42, 0.08)',
            backdropFilter: 'blur(16px)',
          }}
        >
          <div
            style={{
              fontSize: '0.82rem',
              fontWeight: 700,
              letterSpacing: '0.14em',
              textTransform: 'uppercase',
              color: '#0f766e',
            }}
          >
            Browser Demo
          </div>
          <h1
            style={{
              margin: 0,
              fontSize: 'clamp(2rem, 5vw, 4.5rem)',
              lineHeight: 0.95,
              letterSpacing: '-0.04em',
              fontFamily: '"Fraunces", "Iowan Old Style", Georgia, serif',
            }}
          >
            Etch in the browser.
          </h1>
          <p
            style={{
              margin: 0,
              maxWidth: '58rem',
              fontSize: '1.05rem',
              lineHeight: 1.7,
              color: '#334155',
            }}
          >
            This reference app exercises the first React binding over the existing
            Rust/WASM Etch core. It keeps the architecture portable by pushing the
            editor state and preview logic into a framework-agnostic package.
          </p>
        </header>

        <EtchEditor initialSource={INITIAL_SOURCE} wasmUrl={wasmUrl} />
      </div>
    </main>
  );
}
