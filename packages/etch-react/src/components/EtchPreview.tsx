export interface EtchPreviewProps {
  html: string;
  className?: string;
  title?: string;
}

export function EtchPreview({
  html,
  className,
  title = 'Etch preview',
}: EtchPreviewProps) {
  return (
    <iframe
      className={className}
      sandbox=""
      srcDoc={html}
      title={title}
      style={{
        minHeight: 0,
        height: '100%',
        width: '100%',
        border: '1px solid rgba(148, 163, 184, 0.18)',
        borderRadius: '1rem',
        background: '#ffffff',
        boxShadow: '0 18px 40px rgba(15, 23, 42, 0.12)',
      }}
    />
  );
}
