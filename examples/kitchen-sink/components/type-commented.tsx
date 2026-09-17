export type TypeCommentedProps = {
  /** Heading displayed at the top of the card. */
  title: string;
  /** Visual treatment used for the card surface. */
  tone?: "neutral" | "accent" | "positive";
  /** Numeric value shown below the heading. */
  count?: number;
  /** Reduces the visual emphasis of the content. */
  muted?: boolean;
  /** This documented prop is intentionally not destructured at runtime. */
  declaredOnly?: string;
};

export const TypeCommented = ({
  title,
  tone = "accent",
  count = 6,
  muted = false,
}: TypeCommentedProps) => (
  <article
    data-tone={tone}
    style={{
      width: "min(360px, 100%)",
      padding: "24px",
      border: "1px solid #bfdbfe",
      borderRadius: "16px",
      background: tone === "positive" ? "#ecfdf3" : tone === "accent" ? "#eff6ff" : "#ffffff",
      color: muted ? "#71717a" : "#18181b",
      boxShadow: "0 10px 30px rgba(37, 99, 235, 0.09)",
    }}
  >
    <strong>{title}</strong>
    <p style={{ margin: "10px 0 0" }}>Count: {count}</p>
  </article>
);
