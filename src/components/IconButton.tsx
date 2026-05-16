import type { ReactNode } from 'react';

type IconButtonProps = {
  label: string;
  icon: ReactNode;
  onClick: () => void;
  disabled?: boolean;
  title?: string;
  className?: string;
};

export function IconButton({
  label,
  icon,
  onClick,
  disabled = false,
  title,
  className = '',
}: IconButtonProps) {
  return (
    <button
      type="button"
      className={`icon-button ${className}`.trim()}
      aria-label={label}
      title={title ?? label}
      disabled={disabled}
      onClick={onClick}
    >
      <span aria-hidden="true" className="icon-button__glyph">{icon}</span>
    </button>
  );
}
