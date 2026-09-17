import { jsx, jsxs } from "react/jsx-runtime";
import { useMemo, useState } from "react";
import { Toggle } from "@base-ui/react/toggle";
import { ToggleGroup } from "@base-ui/react/toggle-group";
import { TokenTable } from "./TokenGroups.js";
import { groupTypeStyles, typeRoleRows } from "./tokens.js";
function TypographyDocs({ typography }) {
  const [view, setView] = useState("styles");
  const groups = useMemo(() => groupTypeStyles(typography.styles || []), [typography.styles]);
  const roles = useMemo(() => typeRoleRows(typography), [typography]);
  const families = typography.families || [];
  return /* @__PURE__ */ jsxs("div", { className: "type-docs", children: [
    /* @__PURE__ */ jsxs(
      ToggleGroup,
      {
        className: "docs-switch",
        "aria-label": "Typography view",
        value: [view],
        onValueChange: (next) => {
          const selected = Array.isArray(next) ? next[0] : next;
          if (selected) setView(selected);
        },
        children: [
          /* @__PURE__ */ jsx(Toggle, { value: "styles", className: "docs-switch-item", children: "Styles" }),
          /* @__PURE__ */ jsx(Toggle, { value: "tokens", className: "docs-switch-item", children: "Tokens" })
        ]
      }
    ),
    view === "styles" ? /* @__PURE__ */ jsx(TypeStyles, { groups }) : /* @__PURE__ */ jsx(TypeTokens, { roles, families })
  ] });
}
function TypeStyles({ groups }) {
  if (!groups.length) {
    return /* @__PURE__ */ jsx("p", { className: "docs-empty", children: "No type styles in the resolved token set." });
  }
  return /* @__PURE__ */ jsx("div", { className: "type-styles", children: groups.map((group) => /* @__PURE__ */ jsxs("section", { className: "type-group", children: [
    /* @__PURE__ */ jsx("h3", { className: "type-group-name", children: group.name }),
    group.styles.map((style) => /* @__PURE__ */ jsx(
      "p",
      {
        className: "type-sample",
        style: {
          fontSize: style.fontSize || void 0,
          lineHeight: style.lineHeight || void 0,
          fontFamily: style.fontFamily || void 0,
          fontStyle: style.fontStyle || void 0
        },
        children: style.label
      },
      style.id
    ))
  ] }, group.name)) });
}
function TypeTokens({ roles, families }) {
  if (!roles.length && !families.length) {
    return /* @__PURE__ */ jsx("p", { className: "docs-empty", children: "No typography tokens in the resolved token set." });
  }
  return /* @__PURE__ */ jsxs("div", { className: "type-tables", children: [
    roles.length ? /* @__PURE__ */ jsx(TokenTable, { title: "Type roles", rows: roles }) : null,
    families.length ? /* @__PURE__ */ jsx(TokenTable, { title: "Font family", rows: families }) : null
  ] });
}
export {
  TypographyDocs
};
