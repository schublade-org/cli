export function filterBySource(items, source) {
  if (!source) return items;
  return items.filter((item) => item.source === source);
}

export function filterGroups(groups, family, source) {
  return (groups || [])
    .filter((group) => !family || group.id === family)
    .map((group) => ({
      ...group,
      rows: filterBySource(group.rows || [], source),
    }))
    .filter((group) => group.rows.length);
}

export function groupTypeStyles(styles) {
  const groups = [];
  const byName = new Map();
  for (const style of styles) {
    const name = interfaceGroup(style.group);
    if (!byName.has(name)) {
      const group = { name, styles: [] };
      byName.set(name, group);
      groups.push(group);
    }
    byName.get(name).styles.push(style);
  }
  return groups;
}

function interfaceGroup(group) {
  if (group === "Body" || group === "Label") return "Interface";
  return group || "Type";
}

export function typeRoleRows(typography) {
  if (typography.roles?.length) {
    return typography.roles;
  }
  return (typography.styles || []).map((style) => ({
    token: `--text-${style.id}`,
    value: [style.fontSize, style.lineHeight].filter(Boolean).join(" / "),
    source: style.source,
  }));
}
