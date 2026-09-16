export function Button({ label, variant = "primary", size = "md", disabled = false }) {
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
