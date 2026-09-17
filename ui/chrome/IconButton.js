import { jsx, jsxs } from "react/jsx-runtime";
import { Tooltip } from "@base-ui/react/tooltip";
function IconButton({
  label,
  onClick,
  disabled = false,
  active = false,
  className = "icon-btn",
  children
}) {
  const classes = [className, active ? "is-active" : ""].filter(Boolean).join(" ");
  return /* @__PURE__ */ jsxs(Tooltip.Root, { children: [
    /* @__PURE__ */ jsx(
      Tooltip.Trigger,
      {
        className: classes,
        "aria-label": label,
        disabled,
        onClick,
        children
      }
    ),
    /* @__PURE__ */ jsx(Tooltip.Portal, { children: /* @__PURE__ */ jsx(Tooltip.Positioner, { sideOffset: 6, children: /* @__PURE__ */ jsx(Tooltip.Popup, { className: "tooltip", children: label }) }) })
  ] });
}
export {
  IconButton
};
