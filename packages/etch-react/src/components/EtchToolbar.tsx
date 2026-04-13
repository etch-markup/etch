import { BUILTIN_THEMES } from '@etch-markup/etch-editor-core';

const THEME_NAMES = Object.keys(BUILTIN_THEMES);

export interface EtchToolbarProps {
  theme: string;
  isInitializing?: boolean;
  onThemeChange: (theme: string) => void;
  className?: string;
}

export function EtchToolbar({
  theme,
  isInitializing = false,
  onThemeChange,
  className,
}: EtchToolbarProps) {
  return (
    <div
      className={className}
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        gap: '1rem',
        padding: '0.85rem 1rem',
        borderRadius: '1rem',
        border: '1px solid rgba(148, 163, 184, 0.2)',
        background:
          'linear-gradient(135deg, rgba(255,255,255,0.92), rgba(245,247,250,0.9))',
        boxShadow: '0 10px 30px rgba(15, 23, 42, 0.08)',
      }}
    >
      <div>
        <div style={{ fontSize: '0.78rem', letterSpacing: '0.08em', textTransform: 'uppercase', color: '#64748b' }}>
          Preview Theme
        </div>
        <div style={{ fontSize: '1rem', fontWeight: 600, color: '#0f172a' }}>
          {theme}
        </div>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
        {isInitializing ? (
          <span
            style={{
              padding: '0.35rem 0.65rem',
              borderRadius: '999px',
              background: 'rgba(14, 116, 144, 0.1)',
              color: '#0f766e',
              fontSize: '0.85rem',
              fontWeight: 600,
            }}
          >
            Initializing
          </span>
        ) : null}

        <label style={{ display: 'grid', gap: '0.35rem', color: '#334155' }}>
          <span style={{ fontSize: '0.85rem', fontWeight: 600 }}>Theme</span>
          <select
            value={theme}
            onChange={(event) => onThemeChange(event.target.value)}
            style={{
              minWidth: '12rem',
              borderRadius: '0.75rem',
              border: '1px solid rgba(148, 163, 184, 0.45)',
              padding: '0.55rem 0.75rem',
              background: '#ffffff',
              color: '#0f172a',
              font: 'inherit',
            }}
          >
            {THEME_NAMES.map((themeName) => (
              <option key={themeName} value={themeName}>
                {themeName}
              </option>
            ))}
          </select>
        </label>
      </div>
    </div>
  );
}
