import assert from "node:assert/strict";
import { mockWorkshopRequest } from "./plugin.js";

const bootstrap = await request("GET", "/api/bootstrap");
assert.equal(bootstrap.status, 200);
const catalog = JSON.parse(bootstrap.body);
assert.ok(catalog.catalog.stories.length > 0);
assert.equal(catalog.brand.name, catalog.catalog.name);

const rendered = await request("POST", "/api/render", {
  story: "button-ghost",
  values: { label: "Cancel", variant: "ghost", size: "md", disabled: false },
});
assert.equal(rendered.status, 200);
const payload = JSON.parse(rendered.body);
assert.match(payload.html, /Cancel/);
assert.match(payload.code, /Button/);

const missing = await request("POST", "/api/render", {
  story: "does-not-exist",
  values: {},
});
assert.equal(missing.status, 404);

const preview = await request("GET", "/preview");
assert.equal(preview.status, 200);
assert.match(preview.body, /preview-root/);

const css = await request("GET", "/workshop.css");
assert.equal(css.status, 200);
assert.match(css.body, /--sidebar/);

const skipped = await request("GET", "/chrome/main.js");
assert.equal(skipped.handled, false);

console.log("workshop/dev/server.test.mjs ok");

async function request(method, url, body) {
  const chunks = body === undefined ? [] : [Buffer.from(JSON.stringify(body))];
  const req = {
    method,
    url,
    async *[Symbol.asyncIterator]() {
      for (const chunk of chunks) yield chunk;
    },
    on() {},
  };
  const res = {
    statusCode: 0,
    headers: {},
    body: "",
    writeHead(status, headers) {
      this.statusCode = status;
      this.headers = headers ?? {};
    },
    write(chunk) {
      this.body += String(chunk);
    },
    end(chunk) {
      if (chunk) this.body += String(chunk);
    },
  };
  const handled = await mockWorkshopRequest(req, res, url);
  return { handled, status: res.statusCode, body: res.body, headers: res.headers };
}
