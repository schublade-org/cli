import { jsx, jsxs } from "react/jsx-runtime";
function Controls({ story, values, onChange, open }) {
  return /* @__PURE__ */ jsxs("aside", { className: `controls${open ? " is-open" : ""}`, id: "controls", children: [
    /* @__PURE__ */ jsx("header", { className: "controls-head", children: /* @__PURE__ */ jsx("h2", { children: "Controls" }) }),
    /* @__PURE__ */ jsx("p", { className: "controls-desc", children: story ? story.description || "No description for this story." : "Add a story to edit its props from this panel." }),
    /* @__PURE__ */ jsx("div", { className: "controls-rule", role: "separator" }),
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
    ] }, control.id)) })
  ] });
}
function ControlInput({ control, value, onChange }) {
  switch (control.kind) {
    case "select":
      return /* @__PURE__ */ jsx(
        "select",
        {
          id: `ctrl-${control.id}`,
          value: String(value),
          onChange: (event) => onChange(event.target.value),
          children: control.options.map((option) => /* @__PURE__ */ jsx("option", { value: option.value, children: option.label }, option.value))
        }
      );
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
      return /* @__PURE__ */ jsxs("label", { className: "switch", children: [
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
