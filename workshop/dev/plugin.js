import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { bootstrap } from "./bootstrap.js";
import { renderStory } from "./render.js";

const here = dirname(fileURLToPath(import.meta.url));
const ui = join(here, "../../ui");

const UI_FILES = {
  "/workshop.css": ["workshop.css", "text/css; charset=utf-8"],
  "/preview.css": ["preview.css", "text/css; charset=utf-8"],
  "/preview.js": ["preview.js", "text/javascript; charset=utf-8"],
  "/jsx.js": ["jsx.js", "text/javascript; charset=utf-8"],
  "/render.js": ["render.js", "text/javascript; charset=utf-8"],
  "/vendor/react.production.min.js": [
    "vendor/react.production.min.js",
    "text/javascript; charset=utf-8",
  ],
  "/favicon.svg": ["favicon.svg", "image/svg+xml"],
  "/favicon.ico": ["favicon.svg", "image/svg+xml"],
  "/brand/favicon": ["favicon.svg", "image/svg+xml"],
  "/brand/logo": ["favicon.svg", "image/svg+xml"],
};

export function workshopMockPlugin() {
  return {
    name: "schublade-workshop-mock",
    transformIndexHtml(html) {
      const safe = JSON.stringify(bootstrap).replaceAll("<", "\\u003c");
      const block = `<script type="application/json" id="schublade-bootstrap">${safe}</script>`;
      if (html.includes('id="schublade-bootstrap"')) {
        return html;
      }
      return html.replace("</body>", `    ${block}\n  </body>`);
    },
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const url = requestPath(req.url);
        mockWorkshopRequest(req, res, url).then((handled) => {
          if (!handled) next();
        }, next);
      });
    },
  };
}

export async function mockWorkshopRequest(req, res, url) {
  if (url === "/api/bootstrap" && req.method === "GET") {
    sendJson(res, 200, bootstrap);
    return true;
  }

  if (url === "/api/render" && req.method === "POST") {
    const body = await readJsonBody(req);
    const payload = renderStory(bootstrap, body);
    if (!payload) {
      sendJson(res, 404, { error: "unknown story" });
      return true;
    }
    sendJson(res, 200, payload);
    return true;
  }

  if (url === "/api/generation" && req.method === "GET") {
    sendJson(res, 200, { generation: 0 });
    return true;
  }

  if (url === "/api/events" && req.method === "GET") {
    res.writeHead(200, {
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache",
      Connection: "keep-alive",
    });
    res.write(": chrome-dev mock — catalog reload stays on schublade serve\n\n");
    const timer = setInterval(() => {
      res.write(": ping\n\n");
    }, 20000);
    req.on("close", () => clearInterval(timer));
    return true;
  }

  if (url === "/preview" && req.method === "GET") {
    await sendFile(res, join(ui, "preview.html"), "text/html; charset=utf-8");
    return true;
  }

  const asset = UI_FILES[url];
  if (asset && req.method === "GET") {
    await sendFile(res, join(ui, asset[0]), asset[1]);
    return true;
  }

  return false;
}

export function requestPath(raw) {
  const path = String(raw || "/").split("?")[0];
  return path.length > 1 ? path.replace(/\/$/, "") : path;
}

function sendJson(res, status, body) {
  const json = JSON.stringify(body);
  res.writeHead(status, {
    "Content-Type": "application/json; charset=utf-8",
    "Content-Length": Buffer.byteLength(json),
  });
  res.end(json);
}

async function readJsonBody(req) {
  const chunks = [];
  for await (const chunk of req) {
    chunks.push(chunk);
  }
  const raw = Buffer.concat(chunks).toString("utf8").trim();
  if (!raw) return {};
  return JSON.parse(raw);
}

async function sendFile(res, path, type) {
  try {
    const body = await readFile(path);
    res.writeHead(200, {
      "Content-Type": type,
      "Content-Length": body.length,
    });
    res.end(body);
  } catch {
    res.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
    res.end("Not found");
  }
}
