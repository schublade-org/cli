export interface InterfacePlainProps {
  label: string;
  density?: "comfortable" | "compact";
  progress?: number;
  complete?: boolean;
  declaredOnly?: string;
}

export function InterfacePlain({
  label,
  density = "comfortable",
  progress = 40,
  complete = false,
}: InterfacePlainProps) {
  return (
    <section
      style={{
        width: density === "compact" ? "280px" : "340px",
        padding: density === "compact" ? "16px" : "24px",
        border: "1px solid #e4e4e7",
        borderRadius: "16px",
        background: complete ? "#ecfdf3" : "#ffffff",
      }}
    >
      <strong>{label}</strong>
      <div style={{ height: "8px", marginTop: "14px", overflow: "hidden", borderRadius: "999px", background: "#e4e4e7" }}>
        <span style={{ display: "block", width: `${progress}%`, height: "100%", background: complete ? "#16a34a" : "#2563eb" }} />
      </div>
    </section>
  );
}
