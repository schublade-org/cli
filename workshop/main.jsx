import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Tooltip } from "@base-ui/react/tooltip";
import { App, loadBootstrap } from "./App.jsx";

function readInjectedBootstrap() {
  const injected = document.getElementById("schublade-bootstrap");
  if (!injected) return null;
  const text = injected.textContent.trim();
  if (!text) return null;
  return JSON.parse(text);
}

const injected = readInjectedBootstrap();
const root = createRoot(document.getElementById("root"));

if (injected) {
  root.render(
    <StrictMode>
      <Tooltip.Provider delay={200}>
        <App initialBootstrap={injected} staticMode={Boolean(injected.static)} />
      </Tooltip.Provider>
    </StrictMode>
  );
} else {
  loadBootstrap()
    .then(({ bootstrap, staticMode }) => {
      root.render(
        <StrictMode>
          <Tooltip.Provider delay={200}>
            <App initialBootstrap={bootstrap} staticMode={staticMode} />
          </Tooltip.Provider>
        </StrictMode>
      );
    })
    .catch((error) => {
      console.error(error);
      root.render(
        <StrictMode>
          <div className="boot-error">
            <h1>Could not load the workshop catalog.</h1>
            <p>Check schublade.toml and that the catalog still parses.</p>
          </div>
        </StrictMode>
      );
    });
}
