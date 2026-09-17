import assert from "node:assert/strict";
import { groupNav, groupStories, navItem, splitTitle } from "./groups.js";

assert.deepEqual(splitTitle("Button / Ghost"), { group: "Button", item: "Ghost" });
assert.deepEqual(splitTitle("Accordion"), { group: null, item: "Accordion" });

const sections = groupStories([
  { id: "accordion", title: "Accordion", section: "Components", group: null, item: "Accordion" },
  { id: "button", title: "Button / Default", section: "Components", group: "Button", item: "Default" },
  { id: "button-ghost", title: "Button / Ghost", section: "Components", group: "Button", item: "Ghost" },
  { id: "chip", title: "Chip", section: "Components", group: null, item: "Chip" },
]);

assert.equal(sections.length, 1);
assert.equal(sections[0].entries.length, 3);
assert.equal(sections[0].entries[0].type, "story");
assert.equal(sections[0].entries[1].type, "group");
assert.equal(sections[0].entries[1].name, "Button");
assert.equal(sections[0].entries[1].stories.map((story) => navItem(story)).join(","), "Default,Ghost");
assert.equal(sections[0].entries[2].story.id, "chip");

const withPages = groupNav(
  [{ id: "colors", title: "Colors", section: "Foundations" }],
  [
    { id: "accordion", title: "Accordion", section: "Components", group: null, item: "Accordion" },
  ]
);
assert.equal(withPages.map((section) => section.name).join(","), "Foundations,Components");
assert.equal(withPages[0].entries[0].type, "page");
assert.equal(withPages[0].entries[0].page.id, "colors");

console.log("workshop/groups.test.mjs ok");
