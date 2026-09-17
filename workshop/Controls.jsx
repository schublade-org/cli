export function Controls({ story, values, onChange, open }) {
  return (
    <aside className={`controls${open ? " is-open" : ""}`} id="controls">
      <header className="controls-head">
        <h2>Controls</h2>
      </header>
      <p className="controls-desc">
        {story
          ? story.description || "No description for this story."
          : "Add a story to edit its props from this panel."}
      </p>
      <div className="controls-rule" role="separator" />
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
    </aside>
  );
}

function ControlInput({ control, value, onChange }) {
  switch (control.kind) {
    case "select":
      return (
        <select
          id={`ctrl-${control.id}`}
          value={String(value)}
          onChange={(event) => onChange(event.target.value)}
        >
          {control.options.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      );
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
        <label className="switch">
          <input
            id={`ctrl-${control.id}`}
            type="checkbox"
            checked={Boolean(value)}
            onChange={(event) => onChange(event.target.checked)}
          />
          <span />
        </label>
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
