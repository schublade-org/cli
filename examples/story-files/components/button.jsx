export function Button({
  /** Visible text inside the button. */
  label,
  /** Visual treatment of the button. */
  variant = "primary",
  /** Controls the button's dimensions. */
  size = "md",
  /** Prevents interaction when enabled. */
  disabled = false,
}) {
  return (
    <button
      className="btn"
      data-variant={variant}
      data-size={size}
      type="button"
      disabled={disabled}
    >
      {label}
    </button>
  );
}
