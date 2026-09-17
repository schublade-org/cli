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
  IconSun,
} from "@tabler/icons-react";
import { highlight } from "./highlight.js";
import { IconButton } from "./IconButton.jsx";

const VIEWPORTS = [
  { value: "desktop", label: "Desktop width", icon: IconDeviceDesktop },
  { value: "tablet", label: "Tablet width", icon: IconDeviceTablet },
  { value: "mobile", label: "Mobile width", icon: IconDeviceMobile },
];

const THEMES = [
  { value: "light", label: "Light preview", icon: IconSun },
  { value: "dark", label: "Dark preview", icon: IconMoon },
];

export function Inspector({
  tab,
  onTabChange,
  code,
  copied,
  onCopy,
  viewport,
  onViewport,
  mode,
  onMode,
  a11y,
}) {
  const issueCount = a11y.skipped ? 0 : a11y.violations.length;
  const a11yLabel = a11y.skipped
    ? "a11y"
    : issueCount === 0
      ? "a11y"
      : `a11y (${issueCount})`;

  return (
    <section className="inspector" aria-label="Story inspector">
      <Tabs.Root value={tab} onValueChange={onTabChange} className="inspector-tabs">
        <div className="inspector-bar">
          <Tabs.List className="inspector-tablist">
            <Tabs.Tab className="inspector-tab" value="code">
              Code Usage
            </Tabs.Tab>
            <Tabs.Tab
              className={`inspector-tab${issueCount > 0 ? " has-issues" : ""}`}
              value="a11y"
            >
              {a11yLabel}
            </Tabs.Tab>
          </Tabs.List>
          <div className="inspector-tools">
            <Segmented
              label="Canvas width"
              value={viewport}
              onChange={onViewport}
              items={VIEWPORTS}
            />
            <Segmented
              label="Preview theme"
              value={mode}
              onChange={onMode}
              items={THEMES}
            />
          </div>
        </div>
        <Tabs.Panel className="inspector-panel" value="code">
          <div className="code-toolbar">
            <IconButton label={copied ? "Copied" : "Copy usage"} onClick={onCopy}>
              {copied ? <IconCheck size={18} stroke={1.6} /> : <IconCopy size={18} stroke={1.6} />}
            </IconButton>
          </div>
          <pre className="code-body">
            <code dangerouslySetInnerHTML={{ __html: highlight(code) }} />
          </pre>
        </Tabs.Panel>
        <Tabs.Panel className="inspector-panel" value="a11y">
          <A11yPanel a11y={a11y} />
        </Tabs.Panel>
      </Tabs.Root>
    </section>
  );
}

function Segmented({ label, value, onChange, items }) {
  return (
    <ToggleGroup
      className="segmented"
      aria-label={label}
      value={[value]}
      onValueChange={(next) => {
        const selected = Array.isArray(next) ? next[0] : next;
        if (selected) onChange(selected);
      }}
    >
      {items.map((item) => {
        const Icon = item.icon;
        return (
          <Tooltip.Root key={item.value}>
            <Tooltip.Trigger
              render={
                <Toggle
                  value={item.value}
                  className="segmented-item"
                  aria-label={item.label}
                />
              }
            >
              <Icon size={18} stroke={1.6} />
            </Tooltip.Trigger>
            <Tooltip.Portal>
              <Tooltip.Positioner sideOffset={6}>
                <Tooltip.Popup className="tooltip">{item.label}</Tooltip.Popup>
              </Tooltip.Positioner>
            </Tooltip.Portal>
          </Tooltip.Root>
        );
      })}
    </ToggleGroup>
  );
}

function A11yPanel({ a11y }) {
  if (a11y.skipped) {
    return (
      <p className="a11y-empty">Accessibility checks are disabled in schublade.toml.</p>
    );
  }
  if (a11y.violations.length === 0) {
    return <p className="a11y-empty">No issues with the current rules.</p>;
  }
  return (
    <ul className="a11y-list">
      {a11y.violations.map((item, index) => (
        <li key={`${item.rule}-${item.target}-${index}`}>
          {item.label}{" "}
          <span className="a11y-rule">
            {item.rule} · {item.target}
          </span>
        </li>
      ))}
    </ul>
  );
}
