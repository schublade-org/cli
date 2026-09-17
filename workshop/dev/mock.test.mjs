import assert from "node:assert/strict";
import { bootstrap } from "./bootstrap.js";
import { interpolate, renderStory } from "./render.js";

const CONTROL_KINDS = new Set(["text", "select", "boolean", "number"]);
const kinds = new Set();
const stories = bootstrap.catalog.stories;

assert.equal(bootstrap.catalog.name, "Aarau Designsystem");
assert.equal(bootstrap.brand.name, bootstrap.catalog.name);
assert.equal(bootstrap.theme.trigger, "data-attribute");
assert.equal(bootstrap.a11y.enabled, true);
assert.ok(bootstrap.a11y.rules.some((rule) => rule.id === "button-name"));
assert.equal(bootstrap.static, undefined);

assert.ok(stories.find((story) => story.id === "avatar-group"));
assert.ok(stories.find((story) => story.id === "button-ghost"));
assert.equal(
  stories.filter((story) => story.group === "Button").map((story) => story.item).join(","),
  "Default,Ghost,Disabled"
);
assert.ok(stories.some((story) => story.section === "Foundations" && story.controls.length === 0));

for (const story of stories) {
  assert.ok(story.id && story.title && story.item && story.section);
  assert.ok(Array.isArray(story.controls));
  for (const control of story.controls) {
    assert.ok(CONTROL_KINDS.has(control.kind), control.kind);
    kinds.add(control.kind);
    assert.ok(control.id && control.label);
    assert.ok(Object.prototype.hasOwnProperty.call(control, "default"));
    if (control.kind === "select") {
      assert.ok(control.options.length);
    }
  }
}

assert.deepEqual([...kinds].sort(), ["boolean", "number", "select", "text"]);

const rendered = renderStory(bootstrap, {
  story: "button-ghost",
  values: { label: "Abort", variant: "ghost", size: "sm", disabled: false },
});
assert.match(rendered.html, /Abort/);
assert.match(rendered.html, /data-variant="ghost"/);
assert.match(rendered.code, /Abort/);
assert.equal(rendered.react, null);
assert.equal(rendered.title, "Button / Ghost");

const unknown = renderStory(bootstrap, { story: "nope" });
assert.equal(unknown, null);

const a11y = renderStory(bootstrap, { story: "missing-name", values: {} });
assert.match(a11y.html, /<button[^>]*><\/button>/);
assert.match(a11y.html, /<img /);
assert.doesNotMatch(a11y.html, / alt=/);

assert.equal(interpolate("Hi {{name}} {{{flag}}}", { name: "Ada", flag: true }), "Hi Ada true");

console.log("workshop/dev/mock.test.mjs ok");
