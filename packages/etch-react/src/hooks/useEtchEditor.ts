import { useEffect, useRef, useSyncExternalStore } from 'react';
import {
  createEtchEditorCore,
  type EtchEditorCore,
} from '@etch-markup/etch-editor-core';

export interface UseEtchEditorOptions {
  initialSource?: string | undefined;
  theme?: string | undefined;
  wasmUrl?: string | URL | undefined;
}

export function useEtchEditor(options: UseEtchEditorOptions = {}) {
  const coreRef = useRef<EtchEditorCore | null>(null);
  const disposeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  if (!coreRef.current) {
    coreRef.current = createEtchEditorCore(options);
  }

  const core = coreRef.current;
  const state = useSyncExternalStore(
    (listener) => core.subscribe(listener),
    () => core.getState(),
    () => core.getState()
  );

  useEffect(() => {
    if (typeof options.theme === 'string') {
      core.setTheme(options.theme);
    }
  }, [core, options.theme]);

  useEffect(() => {
    if (disposeTimerRef.current) {
      clearTimeout(disposeTimerRef.current);
      disposeTimerRef.current = null;
    }

    return () => {
      disposeTimerRef.current = setTimeout(() => {
        core.dispose();
        if (coreRef.current === core) {
          coreRef.current = null;
        }
        disposeTimerRef.current = null;
      }, 0);
    };
  }, [core]);

  return {
    core,
    state,
    setSource: (source: string) => core.setSource(source),
    setTheme: (theme: string) => core.setTheme(theme),
  };
}
