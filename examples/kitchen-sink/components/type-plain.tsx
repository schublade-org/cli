export type TypePlainProps = {
  title: string;
  tone?: "neutral" | "accent" | "positive";
  count?: number;
  muted?: boolean;
  declaredOnly?: string;
};

export function TypePlain({
  title,
  tone = "neutral",
  count = 3,
  muted = false,
}: TypePlainProps) {
  return (
    <article
      data-tone={tone}
      style={{
        width: "min(360px, 100%)",
        padding: "24px",
        border: "1px solid #e4e4e7",
        borderRadius: "16px",
        background: tone === "accent" ? "#eff6ff" : tone === "positive" ? "#ecfdf3" : "#ffffff",
        color: muted ? "#71717a" : "#18181b",
        boxShadow: "0 10px 30px rgba(24, 24, 27, 0.08)",
      }}
    >
      <strong>{title}</strong>
      <p style={{ margin: "10px 0 0" }}>Count: {count}</p>
    </article>
  );
}
