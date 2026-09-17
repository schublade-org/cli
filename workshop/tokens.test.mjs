import assert from "node:assert/strict";
import { filterBySource, groupTypeStyles, typeRoleRows } from "./tokens.js";

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
assert.equal(roles[0].token, "--type-display-xl");
assert.equal(roles[0].value, "4.5rem");

console.log("workshop/tokens.test.mjs ok");
