import { jsx, jsxs } from "react/jsx-runtime";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Tooltip } from "@base-ui/react/tooltip";
import { App, loadBootstrap } from "./App.js";
function readInjectedBootstrap() {
  const injected2 = document.getElementById("schublade-bootstrap");
  if (!injected2) return null;
  const text = injected2.textContent.trim();
  if (!text) return null;
  return JSON.parse(text);
}
const injected = readInjectedBootstrap();
const root = createRoot(document.getElementById("root"));
if (injected) {
  root.render(
    /* @__PURE__ */ jsx(StrictMode, { children: /* @__PURE__ */ jsx(Tooltip.Provider, { delay: 200, children: /* @__PURE__ */ jsx(App, { initialBootstrap: injected, staticMode: Boolean(injected.static) }) }) })
  );
} else {
  loadBootstrap().then(({ bootstrap, staticMode }) => {
    root.render(
      /* @__PURE__ */ jsx(StrictMode, { children: /* @__PURE__ */ jsx(Tooltip.Provider, { delay: 200, children: /* @__PURE__ */ jsx(App, { initialBootstrap: bootstrap, staticMode }) }) })
    );
  }).catch((error) => {
    console.error(error);
    root.render(
      /* @__PURE__ */ jsx(StrictMode, { children: /* @__PURE__ */ jsxs("div", { className: "boot-error", children: [
        /* @__PURE__ */ jsx("h1", { children: "Could not load the workshop catalog." }),
        /* @__PURE__ */ jsx("p", { children: "Check schublade.toml and that the catalog still parses." })
      ] }) })
    );
  });
}
