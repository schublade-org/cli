(function (root) {
  const PEOPLE = [
    { name: "Lena Meier", initials: "LM", color: "#C4B5A0" },
    { name: "Jonas Keller", initials: "JK", color: "#8FA08C" },
    { name: "Theo Hart", initials: "T", color: "#F97316" },
    { name: "Sofia Nguyen", initials: "SN", color: "#B7A99A" },
    { name: "Amir Rossi", initials: "AR", color: "#9AA7B2" },
    { name: "Pia Hofmann", initials: "PH", color: "#C9B8C4" },
    { name: "Noah Graf", initials: "NG", color: "#A3B18A" },
    { name: "Elena Berg", initials: "EB", color: "#D4A373" },
    { name: "Milo Farid", initials: "MF", color: "#7C93A8" },
    { name: "Ava Kunz", initials: "AK", color: "#C08497" },
    { name: "Rico Steiner", initials: "RS", color: "#8E9A7C" },
    { name: "Noor Salim", initials: "NS", color: "#B08968" },
  ];

  function defaultsFromControls(controls) {
    const values = {};
    for (const control of controls || []) {
      values[control.id] = control.default;
    }
    return values;
  }

  function withAttrTokens(values) {
    const next = Object.assign({}, values);
    for (const [key, value] of Object.entries(values)) {
      if (typeof value !== "boolean") continue;
      let token = null;
      switch (key) {
        case "open":
          token = value ? " open" : "";
          break;
        case "disabled":
          token = value ? " disabled" : "";
          break;
        case "dismissible":
          token = value ? "" : " hidden";
          break;
        default:
          token = null;
      }
      if (token !== null) {
        next[`${key}Attr`] = token;
      }
    }
    return next;
  }

  function jsonToString(value) {
    if (value == null) return "";
    if (typeof value === "string") return value;
    if (typeof value === "boolean" || typeof value === "number") return String(value);
    return JSON.stringify(value);
  }

  function escapeHtml(input) {
    return String(input)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function interpolate(template, values, escape) {
    let output = String(template ?? "");
    for (const [key, value] of Object.entries(values)) {
      const raw = jsonToString(value);
      const replacement = escape ? escapeHtml(raw) : raw;
      output = output.split(`{{${key}}}`).join(replacement);
    }
    return output;
  }

  function overflowFaces(total, visible) {
    if (total === 0 || visible === 0) return [0, null];
    if (total <= visible) return [total, null];
    const faces = Math.max(0, visible - 1);
    return [faces, total - faces];
  }

  function renderAvatarGroup(values) {
    const size = values.size || "md";
    const total = Math.min(PEOPLE.length, Math.max(1, Number(values.total) || 7));
    const visible = Math.min(8, Math.max(1, Number(values.visible) || 4));
    const showCount = values.showCount !== false;
    const [faces, overflow] = overflowFaces(total, visible);
    let items = "";
    for (let i = 0; i < faces; i += 1) {
      const person = PEOPLE[i];
      items += `<span class="ag-item"><span class="ag-initials" style="background:${person.color}" aria-label="${escapeHtml(person.name)}">${escapeHtml(person.initials)}</span></span>`;
    }
    if (overflow != null) {
      items += `<span class="ag-item"><span class="ag-overflow" aria-hidden="true">+${overflow}</span></span>`;
    }
    const caption = showCount ? `<p class="ag-caption">${total} members</p>` : "";
    return `<div class="ag-wrap">
  <div class="ag" data-size="${escapeHtml(size)}" role="img" aria-label="${total} members">${items}</div>
  ${caption}
</div>`;
  }

  function renderStory(story, values) {
    const merged = Object.assign(defaultsFromControls(story.controls), values || {});
    const tokens = withAttrTokens(merged);
    const generator = story.generator || "html";

    switch (generator) {
      case "react": {
        const source = story.component_source;
        if (!source) {
          throw new Error(`story '${story.id}' is missing component source`);
        }
        return {
          html: "",
          code: interpolate(story.code || "", tokens, false),
          react: {
            source,
            exportName: story.component_export || story.component_name || "default",
            props: merged,
          },
        };
      }
      case "avatar-group":
        return {
          html: renderAvatarGroup(tokens),
          code: interpolate(story.code || "", tokens, false),
          react: null,
        };
      case "html": {
        if (!story.template) {
          throw new Error(`story '${story.id}' is missing a template`);
        }
        return {
          html: interpolate(story.template, tokens, true),
          code: interpolate(story.code || "", tokens, false),
          react: null,
        };
      }
      default: {
        const _never = generator;
        throw new Error(`unknown generator '${_never}'`);
      }
    }
  }

  root.SchubladeRender = {
    renderStory,
    interpolate,
    overflowFaces,
  };
})(window);
