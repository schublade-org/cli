import { useMemo, useState } from "react";
import { Collapsible } from "@base-ui/react/collapsible";
import { IconChevronDown } from "./icons.js";
import { groupNav, navItem } from "./groups.js";

export function Sidebar({
  catalogName,
  logoUrl,
  pages = [],
  stories,
  selectedId,
  storyId,
  onSelect,
  open,
  onClose,
}) {
  const activeId = selectedId ?? storyId;
  const sections = useMemo(() => groupNav(pages, stories), [pages, stories]);

  return (
    <aside className={`sidebar${open ? " is-open" : ""}`} id="sidebar">
      <div className="brand">
        {logoUrl ? (
          <img className="brand-logo" src={logoUrl} alt="" />
        ) : (
          <span className="brand-mark" aria-hidden="true" />
        )}
        <span className="brand-name">{catalogName}</span>
      </div>
      <nav className="nav" aria-label="Catalog">
        {pages.length === 0 && stories.length === 0 ? (
          <p className="nav-empty">No pages or stories in this catalog.</p>
        ) : (
          sections.map((section) => (
            <section className="nav-section" key={section.name}>
              <h2 className="nav-label">{section.name}</h2>
              <ul className="nav-list">
                {section.entries.map((entry) => {
                  switch (entry.type) {
                    case "page":
                      return (
                        <li key={entry.page.id}>
                          <button
                            type="button"
                            className={`nav-item${entry.page.id === activeId ? " is-active" : ""}`}
                            onClick={() => {
                              onSelect(entry.page.id);
                              onClose();
                            }}
                          >
                            {entry.page.title}
                          </button>
                        </li>
                      );
                    case "story":
                      return (
                        <li key={entry.story.id}>
                          <button
                            type="button"
                            className={`nav-item${entry.story.id === activeId ? " is-active" : ""}`}
                            onClick={() => {
                              onSelect(entry.story.id);
                              onClose();
                            }}
                          >
                            {navItem(entry.story)}
                          </button>
                        </li>
                      );
                    case "group":
                      return (
                        <NavGroup
                          key={entry.name}
                          entry={entry}
                          storyId={activeId}
                          onSelect={(id) => {
                            onSelect(id);
                            onClose();
                          }}
                        />
                      );
                    default: {
                      const _never = entry;
                      void _never;
                      return null;
                    }
                  }
                })}
              </ul>
            </section>
          ))
        )}
      </nav>
    </aside>
  );
}

function NavGroup({ entry, storyId, onSelect }) {
  const containsActive = entry.stories.some((story) => story.id === storyId);
  const [open, setOpen] = useState(containsActive);

  return (
    <li className="nav-group">
      <Collapsible.Root
        open={open || containsActive}
        onOpenChange={(next) => setOpen(next)}
      >
        <Collapsible.Trigger className={`nav-group-trigger${containsActive ? " is-current" : ""}`}>
          <span>{entry.name}</span>
          <IconChevronDown size={16} stroke={1.6} className="nav-group-chevron" />
        </Collapsible.Trigger>
        <Collapsible.Panel className="nav-group-panel">
          <ul className="nav-sublist">
            {entry.stories.map((story) => (
              <li key={story.id}>
                <button
                  type="button"
                  className={`nav-item nav-subitem${story.id === storyId ? " is-active" : ""}`}
                  onClick={() => onSelect(story.id)}
                >
                  {navItem(story)}
                </button>
              </li>
            ))}
          </ul>
        </Collapsible.Panel>
      </Collapsible.Root>
    </li>
  );
}
