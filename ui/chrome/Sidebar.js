import { jsx, jsxs } from "react/jsx-runtime";
import { useMemo, useState } from "react";
import { Collapsible } from "@base-ui/react/collapsible";
import { IconChevronDown } from "./icons.js";
import { groupNav, navItem } from "./groups.js";
function Sidebar({
  catalogName,
  logoUrl,
  pages = [],
  stories,
  selectedId,
  storyId,
  onSelect,
  open,
  onClose
}) {
  const activeId = selectedId ?? storyId;
  const sections = useMemo(() => groupNav(pages, stories), [pages, stories]);
  return /* @__PURE__ */ jsxs("aside", { className: `sidebar${open ? " is-open" : ""}`, id: "sidebar", children: [
    /* @__PURE__ */ jsxs("div", { className: "brand", children: [
      logoUrl ? /* @__PURE__ */ jsx("img", { className: "brand-logo", src: logoUrl, alt: "" }) : /* @__PURE__ */ jsx("span", { className: "brand-mark", "aria-hidden": "true" }),
      /* @__PURE__ */ jsx("span", { className: "brand-name", children: catalogName })
    ] }),
    /* @__PURE__ */ jsx("nav", { className: "nav", "aria-label": "Catalog", children: pages.length === 0 && stories.length === 0 ? /* @__PURE__ */ jsx("p", { className: "nav-empty", children: "No pages or stories in this catalog." }) : sections.map((section) => /* @__PURE__ */ jsxs("section", { className: "nav-section", children: [
      /* @__PURE__ */ jsx("h2", { className: "nav-label", children: section.name }),
      /* @__PURE__ */ jsx("ul", { className: "nav-list", children: section.entries.map((entry) => {
        switch (entry.type) {
          case "page":
            return /* @__PURE__ */ jsx("li", { children: /* @__PURE__ */ jsx(
              "button",
              {
                type: "button",
                className: `nav-item${entry.page.id === activeId ? " is-active" : ""}`,
                onClick: () => {
                  onSelect(entry.page.id);
                  onClose();
                },
                children: entry.page.title
              }
            ) }, entry.page.id);
          case "story":
            return /* @__PURE__ */ jsx("li", { children: /* @__PURE__ */ jsx(
              "button",
              {
                type: "button",
                className: `nav-item${entry.story.id === activeId ? " is-active" : ""}`,
                onClick: () => {
                  onSelect(entry.story.id);
                  onClose();
                },
                children: navItem(entry.story)
              }
            ) }, entry.story.id);
          case "group":
            return /* @__PURE__ */ jsx(
              NavGroup,
              {
                entry,
                storyId: activeId,
                onSelect: (id) => {
                  onSelect(id);
                  onClose();
                }
              },
              entry.name
            );
          default: {
            const _never = entry;
            void _never;
            return null;
          }
        }
      }) })
    ] }, section.name)) })
  ] });
}
function NavGroup({ entry, storyId, onSelect }) {
  const containsActive = entry.stories.some((story) => story.id === storyId);
  const [open, setOpen] = useState(containsActive);
  return /* @__PURE__ */ jsx("li", { className: "nav-group", children: /* @__PURE__ */ jsxs(
    Collapsible.Root,
    {
      open: open || containsActive,
      onOpenChange: (next) => setOpen(next),
      children: [
        /* @__PURE__ */ jsxs(Collapsible.Trigger, { className: `nav-group-trigger${containsActive ? " is-current" : ""}`, children: [
          /* @__PURE__ */ jsx("span", { children: entry.name }),
          /* @__PURE__ */ jsx(IconChevronDown, { size: 16, stroke: 1.6, className: "nav-group-chevron" })
        ] }),
        /* @__PURE__ */ jsx(Collapsible.Panel, { className: "nav-group-panel", children: /* @__PURE__ */ jsx("ul", { className: "nav-sublist", children: entry.stories.map((story) => /* @__PURE__ */ jsx("li", { children: /* @__PURE__ */ jsx(
          "button",
          {
            type: "button",
            className: `nav-item nav-subitem${story.id === storyId ? " is-active" : ""}`,
            onClick: () => onSelect(story.id),
            children: navItem(story)
          }
        ) }, story.id)) }) })
      ]
    }
  ) });
}
export {
  Sidebar
};
