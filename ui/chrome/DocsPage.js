import { jsx } from "react/jsx-runtime";
import { ColorScales } from "./ColorScales.js";
import { TypographyDocs } from "./TypographyDocs.js";
import { filterBySource } from "./tokens.js";
function DocsPage({ page, tokens }) {
  const colors = tokens?.colors ?? [];
  const typography = tokens?.typography ?? { styles: [], families: [], roles: [] };
  return /* @__PURE__ */ jsx("div", { className: "docs", children: page.blocks.map((block, index) => {
    switch (block.type) {
      case "heading":
        return /* @__PURE__ */ jsx(Heading, { level: block.level, children: block.text }, index);
      case "paragraph":
        return /* @__PURE__ */ jsx("p", { className: "docs-copy", children: block.text }, index);
      case "color-scales":
        return /* @__PURE__ */ jsx(
          ColorScales,
          {
            scales: block.scales ?? filterBySource(colors, block.source)
          },
          index
        );
      case "color-scale":
        return /* @__PURE__ */ jsx(ColorScales, { scales: block.scale ? [block.scale] : [] }, index);
      case "typography":
        return /* @__PURE__ */ jsx(
          TypographyDocs,
          {
            typography: {
              styles: filterBySource(typography.styles || [], block.source),
              families: filterBySource(typography.families || [], block.source),
              roles: filterBySource(typography.roles || [], block.source)
            }
          },
          index
        );
      default: {
        const _never = block.type;
        void _never;
        return null;
      }
    }
  }) });
}
function Heading({ level, children }) {
  switch (level) {
    case 1:
      return /* @__PURE__ */ jsx("h1", { className: "docs-heading", children });
    case 2:
      return /* @__PURE__ */ jsx("h2", { className: "docs-heading", children });
    case 3:
      return /* @__PURE__ */ jsx("h3", { className: "docs-heading", children });
    case 4:
      return /* @__PURE__ */ jsx("h4", { className: "docs-heading", children });
    case 5:
      return /* @__PURE__ */ jsx("h5", { className: "docs-heading", children });
    default:
      return /* @__PURE__ */ jsx("h6", { className: "docs-heading", children });
  }
}
export {
  DocsPage
};
