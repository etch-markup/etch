import { useEtchEditor } from '../hooks/useEtchEditor.js';
import { EtchEditorPane } from './EtchEditorPane.js';
import { EtchErrorBanner } from './EtchErrorBanner.js';
import { EtchPreview } from './EtchPreview.js';
import { EtchToolbar } from './EtchToolbar.js';

export interface EtchEditorProps {
  className?: string;
  initialSource?: string;
  theme?: string;
  wasmUrl?: string | URL;
}

export function EtchEditor({
  className,
  initialSource,
  theme,
  wasmUrl,
}: EtchEditorProps) {
  const { state, setSource, setTheme } = useEtchEditor({
    initialSource,
    theme,
    wasmUrl,
  });

  return (
    <section
      className={className}
      style={{
        display: 'grid',
        gap: '1rem',
      }}
    >
      <EtchToolbar
        theme={state.theme}
        isInitializing={state.isInitializing}
        onThemeChange={setTheme}
      />

      <EtchErrorBanner
        errors={state.errors}
        runtimeError={state.runtimeError}
      />

      <div
        style={{
          display: 'flex',
          flexWrap: 'wrap',
          gap: '1rem',
          minHeight: '70vh',
        }}
      >
        <div style={{ flex: '1 1 28rem', minWidth: 0, minHeight: '32rem' }}>
          <EtchEditorPane value={state.source} onChange={setSource} />
        </div>
        <div style={{ flex: '1 1 28rem', minWidth: 0, minHeight: '32rem' }}>
          <EtchPreview html={state.previewHtml} />
        </div>
      </div>
    </section>
  );
}
