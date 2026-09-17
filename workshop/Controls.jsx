import { useEffect, useRef, useState } from "react";
import { Select } from "@base-ui/react/select";
import { IconCheck, IconChevronDown } from "@tabler/icons-react";
import { ResizeHandle } from "./ResizeHandle.jsx";

const DEFAULT_CONTROLS_WIDTH = 450;
const MIN_CONTROLS_WIDTH = 320;
const MIN_STAGE_WIDTH = 320;

export function Controls({ story, values, onChange, open }) {
  const controlsRef = useRef(null);
  const [width, setWidth] = useState(DEFAULT_CONTROLS_WIDTH);
  const [maxWidth, setMaxWidth] = useState(DEFAULT_CONTROLS_WIDTH * 2);

  useEffect(() => {
    const app = controlsRef.current?.parentElement;
    if (!app) return undefined;

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

  return (
    <aside
      ref={controlsRef}
      className={`controls${open ? " is-open" : ""}`}
      id="controls"
      aria-labelledby="controls-title"
      style={{ "--controls-width": `${width}px` }}
    >
      <ResizeHandle
        className="controls-resize-handle"
        orientation="vertical"
        label="Resize controls panel"
        value={width}
        min={MIN_CONTROLS_WIDTH}
        max={maxWidth}
        getValue={currentWidth}
        onChange={setWidth}
      />
      <div className="controls-intro">
        <header className="controls-head">
          <h2 id="controls-title">Controls</h2>
        </header>
        <p className="controls-desc">
          {story
            ? story.description || "No description for this story."
            : "Add a story to edit its props from this panel."}
        </p>
      </div>
      <div className="control-list">
        {!story ? (
          <p className="controls-empty">No controls until a story exists.</p>
        ) : story.controls.length === 0 ? (
          <p className="controls-empty">This story has no controls.</p>
        ) : (
          story.controls.map((control) => (
            <div className="control-row" key={control.id}>
              <label htmlFor={`ctrl-${control.id}`}>{control.label}</label>
              <ControlInput
                control={control}
                value={values[control.id]}
                onChange={(next) => onChange(control.id, next)}
              />
            </div>
          ))
        )}
      </div>
      {story ? <PropsTable props={story.props || []} /> : null}
    </aside>
  );
}

function PropsTable({ props }) {
  if (props.length === 0) return null;
  return (
    <section className="props-section" aria-labelledby="props-title">
      <h3 id="props-title">Props</h3>
      <div className="props-table-wrap">
        <table className="props-table">
          <caption className="sr-only">Component prop metadata</caption>
          <thead>
            <tr>
              <th scope="col">Prop</th>
              <th scope="col">Type</th>
              <th scope="col">Default</th>
            </tr>
          </thead>
          <tbody>
            {props.map((prop) => (
              <tr key={prop.name}>
                <th scope="row">
                  <code>{prop.name}</code>
                  {prop.description ? <span>{prop.description}</span> : null}
                </th>
                <td>{prop.type || "Unknown"}</td>
                <td><code>{prop.default ?? "null"}</code></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function ControlInput({ control, value, onChange }) {
  switch (control.kind) {
    case "select": {
      const options = control.options.map((option) => ({
        label: option.label,
        value: String(option.value),
      }));
      const selectedValue = String(value ?? control.default ?? options[0]?.value ?? "");
      return (
        <Select.Root
          items={options}
          value={selectedValue}
          onValueChange={(next) => {
            if (next !== null) onChange(next);
          }}
        >
          <Select.Trigger id={`ctrl-${control.id}`} className="control-select-trigger">
            <Select.Value />
            <Select.Icon className="control-select-icon">
              <IconChevronDown aria-hidden="true" size={14} stroke={1.8} />
            </Select.Icon>
          </Select.Trigger>
          <Select.Portal>
            <Select.Positioner
              className="control-select-positioner"
              align="end"
              sideOffset={6}
              alignItemWithTrigger={false}
            >
              <Select.Popup className="control-select-popup">
                <Select.List className="control-select-list">
                  {options.map((option) => (
                    <Select.Item
                      className="control-select-item"
                      key={option.value}
                      value={option.value}
                    >
                      <Select.ItemIndicator className="control-select-indicator">
                        <IconCheck aria-hidden="true" size={14} stroke={2} />
                      </Select.ItemIndicator>
                      <Select.ItemText>{option.label}</Select.ItemText>
                    </Select.Item>
                  ))}
                </Select.List>
              </Select.Popup>
            </Select.Positioner>
          </Select.Portal>
        </Select.Root>
      );
    }
    case "number":
      return (
        <input
          id={`ctrl-${control.id}`}
          type="number"
          min={control.min ?? undefined}
          max={control.max ?? undefined}
          value={String(value ?? "")}
          onChange={(event) => {
            const next = Number(event.target.value);
            onChange(Number.isFinite(next) ? next : control.default);
          }}
        />
      );
    case "boolean":
      return (
        <span className="switch">
          <input
            id={`ctrl-${control.id}`}
            type="checkbox"
            checked={Boolean(value)}
            onChange={(event) => onChange(event.target.checked)}
          />
          <span />
        </span>
      );
    case "text":
      return (
        <input
          id={`ctrl-${control.id}`}
          type="text"
          value={String(value ?? "")}
          onChange={(event) => onChange(event.target.value)}
        />
      );
    default: {
      const _never = control.kind;
      void _never;
      return null;
    }
  }
}
