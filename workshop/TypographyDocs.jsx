import { useMemo, useState } from "react";
import { Toggle } from "@base-ui/react/toggle";
import { ToggleGroup } from "@base-ui/react/toggle-group";
import { TokenTable } from "./TokenGroups.jsx";
import { groupTypeStyles, typeRoleRows } from "./tokens.js";

export function TypographyDocs({ typography }) {
  const [view, setView] = useState("styles");
  const groups = useMemo(() => groupTypeStyles(typography.styles || []), [typography.styles]);
  const roles = useMemo(() => typeRoleRows(typography), [typography]);
  const families = typography.families || [];

  return (
    <div className="type-docs">
      <ToggleGroup
        className="docs-switch"
        aria-label="Typography view"
        value={[view]}
        onValueChange={(next) => {
          const selected = Array.isArray(next) ? next[0] : next;
          if (selected) setView(selected);
        }}
      >
        <Toggle value="styles" className="docs-switch-item">
          Styles
        </Toggle>
        <Toggle value="tokens" className="docs-switch-item">
          Tokens
        </Toggle>
      </ToggleGroup>
      {view === "styles" ? (
        <TypeStyles groups={groups} />
      ) : (
        <TypeTokens roles={roles} families={families} />
      )}
    </div>
  );
}

function TypeStyles({ groups }) {
  if (!groups.length) {
    return <p className="docs-empty">No type styles in the resolved token set.</p>;
  }
  return (
    <div className="type-styles">
      {groups.map((group) => (
        <section className="type-group" key={group.name}>
          <h3 className="type-group-name">{group.name}</h3>
          {group.styles.map((style) => (
            <p
              key={style.id}
              className="type-sample"
              style={{
                fontSize: style.fontSize || undefined,
                lineHeight: style.lineHeight || undefined,
                fontFamily: style.fontFamily || undefined,
                fontStyle: style.fontStyle || undefined,
              }}
            >
              {style.label}
            </p>
          ))}
        </section>
      ))}
    </div>
  );
}

function TypeTokens({ roles, families }) {
  if (!roles.length && !families.length) {
    return <p className="docs-empty">No typography tokens in the resolved token set.</p>;
  }
  return (
    <div className="type-tables">
      {roles.length ? (
        <TokenTable title="Type roles" rows={roles} />
      ) : null}
      {families.length ? (
        <TokenTable title="Font family" rows={families} />
      ) : null}
    </div>
  );
}
