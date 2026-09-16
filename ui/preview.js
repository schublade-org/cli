(function () {
  const root = document.getElementById("root");
  let themeConfig = {
    trigger: "data-attribute",
    key: "data-theme",
    light: "light",
    dark: "dark",
  };
  let a11yConfig = { enabled: true, rules: [] };

  window.addEventListener("message", (event) => {
    const message = event.data;
    if (!message || message.source !== "schublade") {
      return;
    }

    switch (message.type) {
      case "configure":
        themeConfig = message.theme ?? themeConfig;
        a11yConfig = message.a11y ?? a11yConfig;
        applyTheme(message.mode ?? "light");
        break;
      case "render":
        applyTheme(message.mode ?? "light");
        root.innerHTML = message.html ?? "";
        reportA11y();
        break;
      case "theme":
        applyTheme(message.mode ?? "light");
        reportA11y();
        break;
      default: {
        const _never = message.type;
        void _never;
      }
    }
  });

  window.parent.postMessage({ source: "schublade-preview", type: "ready" }, "*");

  function applyTheme(mode) {
    const value = mode === "dark" ? themeConfig.dark : themeConfig.light;
    const rootEl = document.documentElement;

    rootEl.removeAttribute("data-theme");
    rootEl.classList.remove(themeConfig.light, themeConfig.dark, "light", "dark");
    try {
      window.localStorage.removeItem(themeConfig.key);
    } catch (_error) {
      /* sandboxed storage may be opaque */
    }

    switch (themeConfig.trigger) {
      case "class-name":
        rootEl.classList.add(value);
        break;
      case "local-storage":
        try {
          window.localStorage.setItem(themeConfig.key, value);
        } catch (_error) {
          /* ignore */
        }
        rootEl.setAttribute("data-theme", value);
        rootEl.classList.add(value);
        break;
      case "data-attribute":
        rootEl.setAttribute(themeConfig.key || "data-theme", value);
        break;
      default: {
        const _exhaustive = themeConfig.trigger;
        void _exhaustive;
        rootEl.setAttribute("data-theme", value);
      }
    }
  }

  function isVisible(node) {
    return !node.hidden && !node.closest("[hidden]") && node.getAttribute("aria-hidden") !== "true";
  }

  function accessibleName(el) {
    const labelledby = el.getAttribute("aria-labelledby");
    if (labelledby) {
      return labelledby
        .split(/\s+/)
        .map((id) => document.getElementById(id)?.textContent ?? "")
        .join(" ")
        .trim();
    }
    const aria = el.getAttribute("aria-label");
    if (aria && aria.trim()) {
      return aria.trim();
    }
    if (el.id) {
      const label = document.querySelector(`label[for="${cssEscape(el.id)}"]`);
      if (label) {
        return textOf(label);
      }
    }
    const wrapping = el.closest("label");
    if (wrapping) {
      return textOf(wrapping);
    }
    return (el.textContent || el.getAttribute("title") || el.getAttribute("placeholder") || "").trim();
  }

  function textOf(el) {
    return (el.textContent || "").replace(/\s+/g, " ").trim();
  }

  function cssEscape(value) {
    return window.CSS && CSS.escape ? CSS.escape(value) : value.replace(/"/g, '\\"');
  }

  function reportA11y() {
    if (!a11yConfig.enabled) {
      window.parent.postMessage(
        { source: "schublade-preview", type: "a11y", violations: [], skipped: true },
        "*"
      );
      return;
    }

    const ruleIds = new Set((a11yConfig.rules || []).map((rule) => rule.id));
    const violations = [];

    if (ruleIds.has("image-alt")) {
      document.querySelectorAll("img").forEach((img) => {
        if (!isVisible(img)) return;
        if (!img.hasAttribute("alt")) {
          violations.push({
            rule: "image-alt",
            label: "Images must expose an alt attribute",
            target: describe(img),
          });
        }
      });
    }

    if (ruleIds.has("button-name")) {
      document.querySelectorAll("button").forEach((button) => {
        if (!isVisible(button)) return;
        if (!accessibleName(button)) {
          violations.push({
            rule: "button-name",
            label: "Buttons must have an accessible name",
            target: describe(button),
          });
        }
      });
    }

    if (ruleIds.has("link-name")) {
      document.querySelectorAll("a").forEach((link) => {
        if (!isVisible(link)) return;
        if (!accessibleName(link)) {
          violations.push({
            rule: "link-name",
            label: "Links must have an accessible name",
            target: describe(link),
          });
        }
      });
    }

    if (ruleIds.has("label") || ruleIds.has("control-name")) {
      document.querySelectorAll("input, select, textarea").forEach((control) => {
        if (!isVisible(control)) return;
        if (!accessibleName(control)) {
          const rule = ruleIds.has("label") ? "label" : "control-name";
          violations.push({
            rule,
            label:
              rule === "label"
                ? "Form fields must be labelled"
                : "Controls must have an accessible name",
            target: describe(control),
          });
        }
      });
    }

    window.parent.postMessage(
      { source: "schublade-preview", type: "a11y", violations },
      "*"
    );
  }

  function describe(el) {
    const id = el.id ? `#${el.id}` : "";
    const cls = el.className ? `.${String(el.className).trim().split(/\s+/).join(".")}` : "";
    return `${el.tagName.toLowerCase()}${id}${cls}`;
  }
})();
