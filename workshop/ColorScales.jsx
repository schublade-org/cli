import { Tooltip } from "@base-ui/react/tooltip";

export function ColorScales({ scales }) {
  if (!scales.length) {
    return <p className="docs-empty">No color scales in the resolved token set.</p>;
  }
  return (
    <div className="color-scales">
      {scales.map((scale) => (
        <ColorScaleCard key={scale.id} scale={scale} />
      ))}
    </div>
  );
}

function swatchInk(value) {
  const hex = String(value || "").replace("#", "");
  if (hex.length < 6 || hex.split("").some((ch) => Number.isNaN(parseInt(ch, 16)))) {
    return undefined;
  }
  const red = parseInt(hex.slice(0, 2), 16);
  const green = parseInt(hex.slice(2, 4), 16);
  const blue = parseInt(hex.slice(4, 6), 16);
  const luma = (red * 299 + green * 587 + blue * 114) / 1000;
  return luma > 150 ? "#111113" : "#ffffff";
}

function ColorScaleCard({ scale }) {
  return (
    <section className="color-scale">
      <h3 className="color-scale-name">{scale.name}</h3>
      <ol className="color-scale-steps">
        {scale.steps.map((step) => (
          <li key={step.token}>
            <Tooltip.Root>
              <Tooltip.Trigger
                render={
                  <button
                    type="button"
                    className="color-swatch"
                    style={{ background: step.value, color: swatchInk(step.value) }}
                    aria-label={`${scale.id} / ${step.step} ${step.value}`}
                  />
                }
              >
                <span className="color-swatch-step">{step.step}</span>
              </Tooltip.Trigger>
              <Tooltip.Portal>
                <Tooltip.Positioner side="right" sideOffset={10}>
                  <Tooltip.Popup className="token-tip">
                    {scale.id} / {step.step} · {step.value}
                  </Tooltip.Popup>
                </Tooltip.Positioner>
              </Tooltip.Portal>
            </Tooltip.Root>
          </li>
        ))}
      </ol>
    </section>
  );
}
