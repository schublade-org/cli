import { jsx, jsxs } from "react/jsx-runtime";
import { useEffect, useRef, useState } from "react";
import { Select } from "@base-ui/react/select";
import { IconCheck, IconChevronDown } from "@tabler/icons-react";
import { ResizeHandle } from "./ResizeHandle.js";
const DEFAULT_CONTROLS_WIDTH = 450;
const MIN_CONTROLS_WIDTH = 320;
const MIN_STAGE_WIDTH = 320;
function Controls({ story, values, onChange, open }) {
  const controlsRef = useRef(null);
  const [width, setWidth] = useState(DEFAULT_CONTROLS_WIDTH);
  const [maxWidth, setMaxWidth] = useState(DEFAULT_CONTROLS_WIDTH * 2);
  useEffect(() => {
    const app = controlsRef.current?.parentElement;
    if (!app) return void 0;
    const updateMaxWidth = () => {
      if (window.matchMedia("(max-width: 960px)").matches) return;
      const sidebarWidth = app.querySelector(".sidebar")?.getBoundingClientRect().width || 0;
      const nextMax = Math.max(
        MIN_CONTROLS_WIDTH,
        Math.floor(app.getBoundingClientRect().width - sidebarWidth - MIN_STAGE_WIDTH)
      );
      setMaxWidth(nextMax);
      setWidth((current) => Math.min(current, nextMax));
    };
    updateMaxWidth();
    const observer = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(updateMaxWidth);
    observer?.observe(app);
    window.addEventListener("resize", updateMaxWidth);
    return () => {
      observer?.disconnect();
      window.removeEventListener("resize", updateMaxWidth);
    };
  }, []);
  function currentWidth() {
    return width ?? controlsRef.current?.getBoundingClientRect().width ?? DEFAULT_CONTROLS_WIDTH;
  }
  return /* @__PURE__ */ jsxs(
    "aside",
    {
      ref: controlsRef,
      className: `controls${open ? " is-open" : ""}`,
      id: "controls",
      "aria-labelledby": "controls-title",
      style: { "--controls-width": `${width}px` },
      children: [
        /* @__PURE__ */ jsx(
          ResizeHandle,
          {
            className: "controls-resize-handle",
            orientation: "vertical",
            label: "Resize controls panel",
            value: width,
            min: MIN_CONTROLS_WIDTH,
            max: maxWidth,
            getValue: currentWidth,
            onChange: setWidth
          }
        ),
        /* @__PURE__ */ jsxs("div", { className: "controls-intro", children: [
          /* @__PURE__ */ jsx("header", { className: "controls-head", children: /* @__PURE__ */ jsx("h2", { id: "controls-title", children: "Controls" }) }),
          /* @__PURE__ */ jsx("p", { className: "controls-desc", children: story ? story.description || "No description for this story." : "Add a story to edit its props from this panel." })
        ] }),
        /* @__PURE__ */ jsx("div", { className: "control-list", children: !story ? /* @__PURE__ */ jsx("p", { className: "controls-empty", children: "No controls until a story exists." }) : story.controls.length === 0 ? /* @__PURE__ */ jsx("p", { className: "controls-empty", children: "This story has no controls." }) : story.controls.map((control) => /* @__PURE__ */ jsxs("div", { className: "control-row", children: [
          /* @__PURE__ */ jsx("label", { htmlFor: `ctrl-${control.id}`, children: control.label }),
          /* @__PURE__ */ jsx(
            ControlInput,
            {
              control,
              value: values[control.id],
              onChange: (next) => onChange(control.id, next)
            }
          )
        ] }, control.id)) }),
        story ? /* @__PURE__ */ jsx(PropsTable, { props: story.props || [] }) : null
      ]
    }
  );
}
function PropsTable({ props }) {
  if (props.length === 0) return null;
  return /* @__PURE__ */ jsxs("section", { className: "props-section", "aria-labelledby": "props-title", children: [
    /* @__PURE__ */ jsx("h3", { id: "props-title", children: "Props" }),
    /* @__PURE__ */ jsx("div", { className: "props-table-wrap", children: /* @__PURE__ */ jsxs("table", { className: "props-table", children: [
      /* @__PURE__ */ jsx("caption", { className: "sr-only", children: "Component prop metadata" }),
      /* @__PURE__ */ jsx("thead", { children: /* @__PURE__ */ jsxs("tr", { children: [
        /* @__PURE__ */ jsx("th", { scope: "col", children: "Prop" }),
        /* @__PURE__ */ jsx("th", { scope: "col", children: "Type" }),
        /* @__PURE__ */ jsx("th", { scope: "col", children: "Default" })
      ] }) }),
      /* @__PURE__ */ jsx("tbody", { children: props.map((prop) => /* @__PURE__ */ jsxs("tr", { children: [
        /* @__PURE__ */ jsxs("th", { scope: "row", children: [
          /* @__PURE__ */ jsx("code", { children: prop.name }),
          prop.description ? /* @__PURE__ */ jsx("span", { children: prop.description }) : null
        ] }),
        /* @__PURE__ */ jsx("td", { children: prop.type || "Unknown" }),
        /* @__PURE__ */ jsx("td", { children: /* @__PURE__ */ jsx("code", { children: prop.default ?? "null" }) })
      ] }, prop.name)) })
    ] }) })
  ] });
}
function ControlInput({ control, value, onChange }) {
  switch (control.kind) {
    case "select": {
      const options = control.options.map((option) => ({
        label: option.label,
        value: String(option.value)
      }));
      const selectedValue = String(value ?? control.default ?? options[0]?.value ?? "");
      return /* @__PURE__ */ jsxs(
        Select.Root,
        {
          items: options,
          value: selectedValue,
          onValueChange: (next) => {
            if (next !== null) onChange(next);
          },
          children: [
            /* @__PURE__ */ jsxs(Select.Trigger, { id: `ctrl-${control.id}`, className: "control-select-trigger", children: [
              /* @__PURE__ */ jsx(Select.Value, {}),
              /* @__PURE__ */ jsx(Select.Icon, { className: "control-select-icon", children: /* @__PURE__ */ jsx(IconChevronDown, { "aria-hidden": "true", size: 14, stroke: 1.8 }) })
            ] }),
            /* @__PURE__ */ jsx(Select.Portal, { children: /* @__PURE__ */ jsx(
              Select.Positioner,
              {
                className: "control-select-positioner",
                align: "end",
                sideOffset: 6,
                alignItemWithTrigger: false,
                children: /* @__PURE__ */ jsx(Select.Popup, { className: "control-select-popup", children: /* @__PURE__ */ jsx(Select.List, { className: "control-select-list", children: options.map((option) => /* @__PURE__ */ jsxs(
                  Select.Item,
                  {
                    className: "control-select-item",
                    value: option.value,
                    children: [
                      /* @__PURE__ */ jsx(Select.ItemIndicator, { className: "control-select-indicator", children: /* @__PURE__ */ jsx(IconCheck, { "aria-hidden": "true", size: 14, stroke: 2 }) }),
                      /* @__PURE__ */ jsx(Select.ItemText, { children: option.label })
                    ]
                  },
                  option.value
                )) }) })
              }
            ) })
          ]
        }
      );
    }
    case "number":
      return /* @__PURE__ */ jsx(
        "input",
        {
          id: `ctrl-${control.id}`,
          type: "number",
          min: control.min ?? void 0,
          max: control.max ?? void 0,
          value: String(value ?? ""),
          onChange: (event) => {
            const next = Number(event.target.value);
            onChange(Number.isFinite(next) ? next : control.default);
          }
        }
      );
    case "boolean":
      return /* @__PURE__ */ jsxs("span", { className: "switch", children: [
        /* @__PURE__ */ jsx(
          "input",
          {
            id: `ctrl-${control.id}`,
            type: "checkbox",
            checked: Boolean(value),
            onChange: (event) => onChange(event.target.checked)
          }
        ),
        /* @__PURE__ */ jsx("span", {})
      ] });
    case "text":
      return /* @__PURE__ */ jsx(
        "input",
        {
          id: `ctrl-${control.id}`,
          type: "text",
          value: String(value ?? ""),
          onChange: (event) => onChange(event.target.value)
        }
      );
    default: {
      const _never = control.kind;
      void _never;
      return null;
    }
  }
}
export {
  Controls
};
