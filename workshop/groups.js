export function navItem(story) {
  return story.item || splitTitle(story.title).item;
}

export function navGroup(story) {
  if (story.group) return story.group;
  return splitTitle(story.title).group;
}

export function splitTitle(title) {
  const index = String(title || "").indexOf(" / ");
  if (index === -1) {
    return { group: null, item: String(title || "").trim() };
  }
  const group = title.slice(0, index).trim();
  const item = title.slice(index + 3).trim();
  if (!group || !item) {
    return { group: null, item: String(title || "").trim() };
  }
  return { group, item };
}

export function groupStories(stories) {
  return groupNav([], stories);
}

export function groupNav(pages = [], stories = []) {
  const sections = [];
  const bySection = new Map();

  for (const page of pages) {
    const sectionName = page.section || "Foundations";
    if (!bySection.has(sectionName)) {
      const section = { name: sectionName, entries: [] };
      bySection.set(sectionName, section);
      sections.push(section);
    }
    bySection.get(sectionName).entries.push({ type: "page", page });
  }

  for (const story of stories) {
    const sectionName = story.section || "Components";
    if (!bySection.has(sectionName)) {
      const section = { name: sectionName, entries: [] };
      bySection.set(sectionName, section);
      sections.push(section);
    }
    const section = bySection.get(sectionName);
    const group = navGroup(story);
    if (!group) {
      section.entries.push({ type: "story", story });
      continue;
    }
    const last = section.entries[section.entries.length - 1];
    if (last && last.type === "group" && last.name === group) {
      last.stories.push(story);
      continue;
    }
    section.entries.push({ type: "group", name: group, stories: [story] });
  }

  return sections;
}
