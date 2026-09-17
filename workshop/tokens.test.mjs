import assert from "node:assert/strict";
import { filterBySource, filterGroups, groupTypeStyles, typeRoleRows } from "./tokens.js";

const styles = [
  { id: "display-xl", label: "Display / xl", group: "Display", fontSize: "4.5rem" },
  { id: "body-lg", label: "Body / lg", group: "Body", fontSize: "1.125rem", source: "css" },
  { id: "label-sm", label: "Label / sm", group: "Label", fontSize: "0.8rem", source: "manual" },
];

const groups = groupTypeStyles(styles);
assert.equal(groups.map((group) => group.name).join(","), "Display,Interface");
assert.equal(groups[1].styles.map((style) => style.id).join(","), "body-lg,label-sm");

assert.equal(filterBySource(styles, "manual").length, 1);
assert.equal(filterBySource(styles, null).length, 3);

const roles = typeRoleRows({
  styles,
  roles: [],
  families: [],
});
assert.equal(roles[0].token, "--text-display-xl");
assert.equal(roles[0].value, "4.5rem");

const tokenGroups = filterGroups(
  [
    { id: "spacing", name: "Spacing", rows: [{ token: "--spacing-4", value: "1rem", source: "css" }] },
    { id: "radius", name: "Radius", rows: [{ token: "--radius-md", value: "0.375rem", source: "manual" }] },
  ],
  "spacing",
  "css"
);
assert.equal(tokenGroups.length, 1);
assert.equal(tokenGroups[0].id, "spacing");

console.log("workshop/tokens.test.mjs ok");
