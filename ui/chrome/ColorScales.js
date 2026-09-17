import { jsx, jsxs } from "react/jsx-runtime";
import { Tooltip } from "@base-ui/react/tooltip";
function ColorScales({ scales }) {
  if (!scales.length) {
    return /* @__PURE__ */ jsx("p", { className: "docs-empty", children: "No color scales in the resolved token set." });
  }
  return /* @__PURE__ */ jsx("div", { className: "color-scales", children: scales.map((scale) => /* @__PURE__ */ jsx(ColorScaleCard, { scale }, scale.id)) });
}
function swatchInk(value) {
  const hex = String(value || "").replace("#", "");
  if (hex.length < 6 || hex.split("").some((ch) => Number.isNaN(parseInt(ch, 16)))) {
    return void 0;
  }
  const red = parseInt(hex.slice(0, 2), 16);
  const green = parseInt(hex.slice(2, 4), 16);
  const blue = parseInt(hex.slice(4, 6), 16);
  const luma = (red * 299 + green * 587 + blue * 114) / 1e3;
  return luma > 150 ? "#111113" : "#ffffff";
}
function ColorScaleCard({ scale }) {
  return /* @__PURE__ */ jsxs("section", { className: "color-scale", children: [
    /* @__PURE__ */ jsx("h3", { className: "color-scale-name", children: scale.name }),
    /* @__PURE__ */ jsx("ol", { className: "color-scale-steps", children: scale.steps.map((step) => /* @__PURE__ */ jsx("li", { children: /* @__PURE__ */ jsxs(Tooltip.Root, { children: [
      /* @__PURE__ */ jsx(
        Tooltip.Trigger,
        {
          render: /* @__PURE__ */ jsx(
            "button",
            {
              type: "button",
              className: "color-swatch",
              style: { background: step.value, color: swatchInk(step.value) },
              "aria-label": `${scale.id} / ${step.step} ${step.value}`
            }
          ),
          children: /* @__PURE__ */ jsx("span", { className: "color-swatch-step", children: step.step })
        }
      ),
      /* @__PURE__ */ jsx(Tooltip.Portal, { children: /* @__PURE__ */ jsx(Tooltip.Positioner, { side: "right", sideOffset: 10, children: /* @__PURE__ */ jsxs(Tooltip.Popup, { className: "token-tip", children: [
        scale.id,
        " / ",
        step.step,
        " \xB7 ",
        step.value
      ] }) }) })
    ] }) }, step.token)) })
  ] });
}
export {
  ColorScales
};
