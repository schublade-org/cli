export interface InterfaceCommentedProps {
  /** Short name for the tracked task. */
  label: string;
  /** Controls the amount of padding inside the card. */
  density?: "comfortable" | "compact";
  /** Completion percentage from zero through one hundred. */
  progress?: number;
  /** Marks the task as finished. */
  complete?: boolean;
  /** Documented declaration retained even though the component does not read it. */
  declaredOnly?: string;
}

export function InterfaceCommented({
  label,
  density = "compact",
  progress = 72,
  complete = true,
}: InterfaceCommentedProps) {
  return (
    <section
      style={{
        width: density === "compact" ? "280px" : "340px",
        padding: density === "compact" ? "16px" : "24px",
        border: "1px solid #bbf7d0",
        borderRadius: "16px",
        background: complete ? "#ecfdf3" : "#ffffff",
      }}
    >
      <strong>{label}</strong>
      <div style={{ height: "8px", marginTop: "14px", overflow: "hidden", borderRadius: "999px", background: "#dcfce7" }}>
        <span style={{ display: "block", width: `${progress}%`, height: "100%", background: "#16a34a" }} />
      </div>
    </section>
  );
}
