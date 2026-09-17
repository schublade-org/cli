/** Mock /api/render. Same JSON the chrome already consumes from serve. */

export function renderStory(bootstrap, request) {
  const id = request?.story;
  const story = bootstrap.catalog.stories.find((item) => item.id === id);
  if (!story) return null;

  const values = defaultsFromControls(story.controls);
  const incoming = request.values && typeof request.values === "object" ? request.values : {};
  for (const [key, value] of Object.entries(incoming)) {
    values[key] = value;
  }

  const tokens = withAttrTokens(values);
  return {
    html: interpolate(story.template || "", tokens),
    code: interpolate(story.code || "", tokens),
    react: null,
    title: story.title,
    description: story.description,
  };
}

export function defaultsFromControls(controls) {
  const values = {};
  for (const control of controls) {
    values[control.id] = control.default;
  }
  return values;
}

export function withAttrTokens(values) {
  const next = { ...values };
  if (typeof values.open === "boolean") {
    next.openAttr = values.open ? " open" : "";
  }
  if (typeof values.disabled === "boolean") {
    next.disabledAttr = values.disabled ? " disabled" : "";
  }
  if (typeof values.dismissible === "boolean") {
    next.dismissAttr = values.dismissible ? "" : " hidden";
  }
  return next;
}

export function interpolate(template, values) {
  return String(template)
    .replaceAll(/\{\{\{(\w+)\}\}\}/g, (_, key) => String(values[key] ?? ""))
    .replaceAll(/\{\{(\w+)\}\}/g, (_, key) => String(values[key] ?? ""));
}
