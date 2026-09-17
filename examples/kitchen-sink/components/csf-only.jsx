export function CsfOnly({ label = "CSF only", emphasis = "quiet" }) {
  return (
    <div
      style={{
        padding: "20px 24px",
        border: emphasis === "strong" ? "2px solid #2563eb" : "1px solid #d4d4d8",
        borderRadius: "14px",
        background: emphasis === "strong" ? "#eff6ff" : "#ffffff",
      }}
    >
      <strong>{label}</strong>
      <p style={{ margin: "8px 0 0", color: "#71717a" }}>No TOML story entry backs this card.</p>
    </div>
  );
}
