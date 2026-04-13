import type { ParseError } from '@etch-markup/etch-editor-core';

export interface EtchErrorBannerProps {
  errors: ParseError[];
  runtimeError?: string | undefined;
  className?: string | undefined;
}

export function EtchErrorBanner({
  errors,
  runtimeError,
  className,
}: EtchErrorBannerProps) {
  if (!runtimeError && errors.length === 0) {
    return null;
  }

  const hasErrors = errors.some((error) => error.kind === 'Error');
  const tone = runtimeError || hasErrors ? '#991b1b' : '#92400e';
  const background = runtimeError || hasErrors
    ? 'rgba(254, 226, 226, 0.92)'
    : 'rgba(254, 243, 199, 0.92)';
  const border = runtimeError || hasErrors
    ? 'rgba(248, 113, 113, 0.4)'
    : 'rgba(245, 158, 11, 0.4)';

  return (
    <div
      className={className}
      style={{
        display: 'grid',
        gap: '0.6rem',
        padding: '0.9rem 1rem',
        borderRadius: '1rem',
        border: `1px solid ${border}`,
        background,
        color: tone,
        boxShadow: '0 10px 30px rgba(15, 23, 42, 0.06)',
      }}
    >
      {runtimeError ? (
        <div>
          <strong>Runtime error.</strong> {runtimeError}
        </div>
      ) : null}

      {errors.length > 0 ? (
        <div style={{ display: 'grid', gap: '0.45rem' }}>
          <strong>
            {hasErrors ? 'Parser reported errors.' : 'Parser reported warnings.'}
          </strong>
          <ul style={{ margin: 0, paddingLeft: '1.2rem' }}>
            {errors.map((error, index) => {
              const location = typeof error.column === 'number'
                ? `Line ${error.line}, column ${error.column}`
                : `Line ${error.line}`;

              return (
                <li key={`${location}-${index}`}>
                  <strong>{location}</strong>: {error.message}
                </li>
              );
            })}
          </ul>
        </div>
      ) : null}
    </div>
  );
}
