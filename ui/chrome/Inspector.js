import { jsx, jsxs } from "react/jsx-runtime";
import { useEffect, useRef, useState } from "react";
import { Tabs } from "@base-ui/react/tabs";
import { Toggle } from "@base-ui/react/toggle";
import { ToggleGroup } from "@base-ui/react/toggle-group";
import { Tooltip } from "@base-ui/react/tooltip";
import {
  IconCheck,
  IconCopy,
  IconDeviceDesktop,
  IconDeviceMobile,
  IconDeviceTablet,
  IconMoon,
  IconSun
} from "./icons.js";
import { highlight } from "./highlight.js";
import { IconButton } from "./IconButton.js";
import { ResizeHandle } from "./ResizeHandle.js";
const VIEWPORTS = [
  { value: "desktop", label: "Desktop width", icon: IconDeviceDesktop },
  { value: "tablet", label: "Tablet width", icon: IconDeviceTablet },
  { value: "mobile", label: "Mobile width", icon: IconDeviceMobile }
];
const THEMES = [
  { value: "light", label: "Light preview", icon: IconSun },
  { value: "dark", label: "Dark preview", icon: IconMoon }
];
const MIN_INSPECTOR_HEIGHT = 96;
const MIN_CANVAS_HEIGHT = 96;
function Inspector({
  tab,
  onTabChange,
  code,
  copied,
  onCopy,
  viewport,
  onViewport,
  mode,
  onMode,
  a11y
}) {
  const inspectorRef = useRef(null);
  const [height, setHeight] = useState(null);
  const [maxHeight, setMaxHeight] = useState(MIN_INSPECTOR_HEIGHT * 6);
  const issueCount = a11y.skipped ? 0 : a11y.violations.length;
  const a11yLabel = a11y.skipped ? "a11y" : issueCount === 0 ? "a11y" : `a11y (${issueCount})`;
  useEffect(() => {
    const stage = inspectorRef.current?.parentElement;
    if (!stage) return void 0;
    const updateMaxHeight = () => {
      const headerHeight = stage.querySelector(".stage-head")?.getBoundingClientRect().height || 0;
      const nextMax = Math.max(
        MIN_INSPECTOR_HEIGHT,
        Math.floor(stage.getBoundingClientRect().height - headerHeight - MIN_CANVAS_HEIGHT)
      );
      setMaxHeight(nextMax);
      setHeight((current) => current === null ? null : Math.min(current, nextMax));
    };
    updateMaxHeight();
    window.addEventListener("resize", updateMaxHeight);
    return () => window.removeEventListener("resize", updateMaxHeight);
  }, []);
  function currentHeight() {
    return height ?? inspectorRef.current?.getBoundingClientRect().height ?? MIN_INSPECTOR_HEIGHT;
  }
  return /* @__PURE__ */ jsxs(
    "section",
    {
      ref: inspectorRef,
      className: `inspector${height === null ? "" : " is-resized"}`,
      "aria-label": "Story inspector",
      style: height === null ? void 0 : { height: `${height}px` },
      children: [
        /* @__PURE__ */ jsx(
          ResizeHandle,
          {
            className: "inspector-resize-handle",
            orientation: "horizontal",
            label: "Resize inspector panel",
            value: height,
            min: MIN_INSPECTOR_HEIGHT,
            max: maxHeight,
            getValue: currentHeight,
            onChange: setHeight
          }
        ),
        /* @__PURE__ */ jsxs(Tabs.Root, { value: tab, onValueChange: onTabChange, className: "inspector-tabs", children: [
          /* @__PURE__ */ jsxs("div", { className: "inspector-bar", children: [
            /* @__PURE__ */ jsxs(Tabs.List, { className: "inspector-tablist", children: [
              /* @__PURE__ */ jsx(Tabs.Tab, { className: "inspector-tab", value: "code", children: "Code Usage" }),
              /* @__PURE__ */ jsx(
                Tabs.Tab,
                {
                  className: `inspector-tab${issueCount > 0 ? " has-issues" : ""}`,
                  value: "a11y",
                  children: a11yLabel
                }
              )
            ] }),
            /* @__PURE__ */ jsxs("div", { className: "inspector-tools", children: [
              /* @__PURE__ */ jsx(
                Segmented,
                {
                  label: "Canvas width",
                  value: viewport,
                  onChange: onViewport,
                  items: VIEWPORTS
                }
              ),
              /* @__PURE__ */ jsx(
                Segmented,
                {
                  label: "Preview theme",
                  value: mode,
                  onChange: onMode,
                  items: THEMES
                }
              )
            ] })
          ] }),
          /* @__PURE__ */ jsxs(Tabs.Panel, { className: "inspector-panel", value: "code", children: [
            /* @__PURE__ */ jsx("div", { className: "code-toolbar", children: /* @__PURE__ */ jsx(IconButton, { label: copied ? "Copied" : "Copy usage", onClick: onCopy, children: copied ? /* @__PURE__ */ jsx(IconCheck, { size: 18, stroke: 1.6 }) : /* @__PURE__ */ jsx(IconCopy, { size: 18, stroke: 1.6 }) }) }),
            /* @__PURE__ */ jsx("pre", { className: "code-body", children: /* @__PURE__ */ jsx("code", { dangerouslySetInnerHTML: { __html: highlight(code) } }) })
          ] }),
          /* @__PURE__ */ jsx(Tabs.Panel, { className: "inspector-panel", value: "a11y", children: /* @__PURE__ */ jsx(A11yPanel, { a11y }) })
        ] })
      ]
    }
  );
}
function Segmented({ label, value, onChange, items }) {
  return /* @__PURE__ */ jsx(
    ToggleGroup,
    {
      className: "segmented",
      "aria-label": label,
      value: [value],
      onValueChange: (next) => {
        const selected = Array.isArray(next) ? next[0] : next;
        if (selected) onChange(selected);
      },
      children: items.map((item) => {
        const Icon = item.icon;
        return /* @__PURE__ */ jsxs(Tooltip.Root, { children: [
          /* @__PURE__ */ jsx(
            Tooltip.Trigger,
            {
              render: /* @__PURE__ */ jsx(
                Toggle,
                {
                  value: item.value,
                  className: "segmented-item",
                  "aria-label": item.label
                }
              ),
              children: /* @__PURE__ */ jsx(Icon, { size: 18, stroke: 1.6 })
            }
          ),
          /* @__PURE__ */ jsx(Tooltip.Portal, { children: /* @__PURE__ */ jsx(Tooltip.Positioner, { sideOffset: 6, children: /* @__PURE__ */ jsx(Tooltip.Popup, { className: "tooltip", children: item.label }) }) })
        ] }, item.value);
      })
    }
  );
}
function A11yPanel({ a11y }) {
  if (a11y.skipped) {
    return /* @__PURE__ */ jsx("p", { className: "a11y-empty", children: "Accessibility checks are disabled in schublade.toml." });
  }
  if (a11y.violations.length === 0) {
    return /* @__PURE__ */ jsx("p", { className: "a11y-empty", children: "No issues with the current rules." });
  }
  return /* @__PURE__ */ jsx("ul", { className: "a11y-list", children: a11y.violations.map((item, index) => /* @__PURE__ */ jsxs("li", { children: [
    item.label,
    " ",
    /* @__PURE__ */ jsxs("span", { className: "a11y-rule", children: [
      item.rule,
      " \xB7 ",
      item.target
    ] })
  ] }, `${item.rule}-${item.target}-${index}`)) });
}
export {
  Inspector
};
