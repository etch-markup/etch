import { useEtchEditor } from '../hooks/useEtchEditor.js';
import { EtchEditorPane } from './EtchEditorPane.js';
import { EtchErrorBanner } from './EtchErrorBanner.js';
import { EtchPreview } from './EtchPreview.js';
import { EtchToolbar } from './EtchToolbar.js';

export interface EtchEditorProps {
  className?: string;
  initialSource?: string;
  value?: string;
  onChange?: (source: string) => void;
  theme?: string;
  wasmUrl?: string | URL;
}

export function EtchEditor({
  className,
  initialSource,
  value,
  onChange,
  theme,
  wasmUrl,
}: EtchEditorProps) {
  const { state, setSource, setTheme } = useEtchEditor({
    initialSource,
    value,
    theme,
    wasmUrl,
  });

  function handleSourceChange(source: string): void {
    setSource(source);
    onChange?.(source);
  }

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
          <EtchEditorPane value={state.source} onChange={handleSourceChange} />
        </div>
        <div style={{ flex: '1 1 28rem', minWidth: 0, minHeight: '32rem' }}>
          <EtchPreview html={state.previewHtml} />
        </div>
      </div>
    </section>
  );
}
