import { jsx, jsxs } from "react/jsx-runtime";
function TokenGroups({ groups }) {
  if (!groups.length) {
    return /* @__PURE__ */ jsx("p", { className: "docs-empty", children: "No tokens in this family." });
  }
  return /* @__PURE__ */ jsx("div", { className: "type-tables", children: groups.map((group) => /* @__PURE__ */ jsx(TokenTable, { title: group.name, rows: group.rows || [] }, group.id)) });
}
function TokenTable({ title, rows }) {
  return /* @__PURE__ */ jsxs("section", { className: "token-table-wrap", children: [
    /* @__PURE__ */ jsx("h3", { className: "token-table-title", children: title }),
    /* @__PURE__ */ jsxs("table", { className: "token-table", children: [
      /* @__PURE__ */ jsx("thead", { children: /* @__PURE__ */ jsxs("tr", { children: [
        /* @__PURE__ */ jsx("th", { children: "Token" }),
        /* @__PURE__ */ jsx("th", { children: "Value" })
      ] }) }),
      /* @__PURE__ */ jsx("tbody", { children: rows.map((row) => /* @__PURE__ */ jsxs("tr", { children: [
        /* @__PURE__ */ jsx("td", { children: row.token }),
        /* @__PURE__ */ jsx("td", { children: row.value })
      ] }, row.token)) })
    ] })
  ] });
}
export {
  TokenGroups,
  TokenTable
};
