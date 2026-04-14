import { useEffect, useRef } from 'react';
import { EditorState } from '@codemirror/state';
import { EditorView } from '@codemirror/view';

export interface EtchEditorPaneProps {
  value: string;
  onChange: (value: string) => void;
  className?: string;
}

export function EtchEditorPane({
  value,
  onChange,
  className,
}: EtchEditorPaneProps) {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const viewRef = useRef<EditorView | null>(null);
  const onChangeRef = useRef(onChange);
  const isApplyingExternalValueRef = useRef(false);

  onChangeRef.current = onChange;

  useEffect(() => {
    if (!hostRef.current) {
      return;
    }

    const view = new EditorView({
      state: EditorState.create({
        doc: value,
        extensions: [
          EditorView.lineWrapping,
          EditorView.updateListener.of((update) => {
            if (!update.docChanged) {
              return;
            }

            if (isApplyingExternalValueRef.current) {
              return;
            }

            onChangeRef.current(update.state.doc.toString());
          }),
          EditorView.contentAttributes.of({
            'aria-label': 'Etch source editor',
            spellcheck: 'false',
          }),
          EditorView.theme({
            '&': {
              height: '100%',
              color: '#f5f7fb',
              backgroundColor: '#0f172a',
              fontSize: '14px',
              fontFamily:
                '"Iosevka Term", "SFMono-Regular", Consolas, "Liberation Mono", monospace',
            },
            '.cm-scroller': {
              overflow: 'auto',
              lineHeight: '1.65',
            },
            '.cm-content': {
              minHeight: '100%',
              padding: '1rem 1.1rem 2rem',
              caretColor: '#f8fafc',
            },
            '.cm-focused': {
              outline: 'none',
            },
            '.cm-gutters': {
              backgroundColor: '#111827',
              borderRight: '1px solid rgba(148, 163, 184, 0.18)',
              color: '#64748b',
            },
            '.cm-activeLineGutter': {
              backgroundColor: 'rgba(30, 41, 59, 0.72)',
            },
            '.cm-activeLine': {
              backgroundColor: 'rgba(30, 41, 59, 0.5)',
            },
            '.cm-selectionBackground, ::selection': {
              backgroundColor: 'rgba(96, 165, 250, 0.28) !important',
            },
          }),
        ],
      }),
      parent: hostRef.current,
    });

    viewRef.current = view;

    return () => {
      view.destroy();
      viewRef.current = null;
    };
  }, []);

  useEffect(() => {
    const view = viewRef.current;
    if (!view) {
      return;
    }

    const currentValue = view.state.doc.toString();
    if (currentValue === value) {
      return;
    }

    isApplyingExternalValueRef.current = true;
    view.dispatch({
      changes: {
        from: 0,
        to: currentValue.length,
        insert: value,
      },
    });
    isApplyingExternalValueRef.current = false;
  }, [value]);

  return (
    <div
      className={className}
      style={{
        minHeight: 0,
        height: '100%',
        borderRadius: '1rem',
        overflow: 'hidden',
        border: '1px solid rgba(148, 163, 184, 0.18)',
        boxShadow: '0 18px 40px rgba(15, 23, 42, 0.18)',
        background: '#0f172a',
      }}
    >
      <div ref={hostRef} style={{ height: '100%' }} />
    </div>
  );
}
