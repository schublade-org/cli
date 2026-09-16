(function () {
  const nav = document.getElementById("nav");
  const catalogName = document.getElementById("catalog-name");
  const storyTitle = document.getElementById("story-title");
  const storyDesc = document.getElementById("story-desc");
  const controlList = document.getElementById("control-list");
  const preview = document.getElementById("preview");
  const canvasFrame = document.getElementById("canvas-frame");
  const canvasStatus = document.getElementById("canvas-status");
  const codeBody = document.getElementById("code-body").querySelector("code");
  const copyCode = document.getElementById("copy-code");
  const themeToggle = document.getElementById("theme-toggle");
  const a11yToggle = document.getElementById("a11y-toggle");
  const a11yPanel = document.getElementById("a11y-panel");
  const a11yEmpty = document.getElementById("a11y-empty");
  const a11yList = document.getElementById("a11y-list");
  const sidebar = document.getElementById("sidebar");
  const controls = document.getElementById("controls");
  const scrim = document.getElementById("scrim");
  const prevStory = document.getElementById("prev-story");
  const nextStory = document.getElementById("next-story");

  const state = {
    bootstrap: null,
    storyId: null,
    values: {},
    mode: localStorage.getItem("schublade:theme") || "light",
    previewReady: false,
    pending: null,
  };

  prevStory.addEventListener("click", () => stepStory(-1));
  nextStory.addEventListener("click", () => stepStory(1));
  document.getElementById("open-nav").addEventListener("click", () => toggleDrawer(sidebar));
  document.getElementById("open-controls").addEventListener("click", () => toggleDrawer(controls));
  scrim.addEventListener("click", closeDrawers);
  copyCode.addEventListener("click", copyUsage);
  themeToggle.addEventListener("click", toggleTheme);
  a11yToggle.addEventListener("click", () => {
    const open = a11yPanel.hasAttribute("hidden");
    a11yPanel.toggleAttribute("hidden", !open);
    a11yToggle.classList.toggle("is-open", open);
    a11yToggle.setAttribute("aria-expanded", String(open));
  });

  document.querySelectorAll("[data-viewport]").forEach((button) => {
    button.addEventListener("click", () => setViewport(button.dataset.viewport));
  });

  window.addEventListener("hashchange", () => {
    if (!state.bootstrap) return;
    const stories = state.bootstrap.catalog.stories;
    if (!stories.length) return;
    selectStory(storyIdFromHash() || stories[0].id, false);
  });

  window.addEventListener("message", (event) => {
    const message = event.data;
    if (!message || message.source !== "schublade-preview") return;

    switch (message.type) {
      case "ready":
        state.previewReady = true;
        configurePreview();
        if (state.pending) {
          sendRender(state.pending);
        }
        break;
      case "a11y":
        renderA11y(message.violations || [], Boolean(message.skipped));
        break;
      default: {
        const _never = message.type;
        void _never;
      }
    }
  });

  boot().catch((error) => {
    showStatus("Could not load the workshop catalog.");
    console.error(error);
  });

  async function boot() {
    showStatus("Loading story…");
    const response = await fetch("/api/bootstrap");
    if (!response.ok) {
      throw new Error("bootstrap failed");
    }
    state.bootstrap = await response.json();
    catalogName.textContent = state.bootstrap.catalog.name;
    document.title = `${state.bootstrap.catalog.name} · Schublade`;
    renderNav();
    const stories = state.bootstrap.catalog.stories;
    if (!stories.length) {
      showEmptyCatalog();
      return;
    }
    setStoryNavEnabled(true);
    const initial =
      storyIdFromHash() ||
      stories.find((story) => story.id === "avatar-group")?.id ||
      stories[0].id;
    await selectStory(initial, true);
  }

  function showEmptyCatalog() {
    setStoryNavEnabled(false);
    storyTitle.textContent = "No stories";
    storyDesc.textContent =
      "This catalog is empty. Add *.stories.toml files or [[stories]] in catalog.toml and restart the server.";
    controlList.innerHTML = "";
    const empty = document.createElement("p");
    empty.className = "controls-empty";
    empty.textContent = "No controls until a story exists.";
    controlList.appendChild(empty);
    codeBody.textContent = "";
    showStatus("This catalog has no stories yet.");
  }

  function setStoryNavEnabled(enabled) {
    prevStory.disabled = !enabled;
    nextStory.disabled = !enabled;
  }

  function renderNav() {
    const stories = state.bootstrap.catalog.stories;
    nav.innerHTML = "";
    if (!stories.length) {
      const empty = document.createElement("p");
      empty.className = "nav-empty";
      empty.textContent = "No stories in this catalog.";
      nav.appendChild(empty);
      return;
    }

    const sections = new Map();
    for (const story of stories) {
      if (!sections.has(story.section)) {
        sections.set(story.section, []);
      }
      sections.get(story.section).push(story);
    }

    for (const [name, sectionStories] of sections) {
      const section = document.createElement("section");
      section.className = "nav-section";
      section.innerHTML = `<h2 class="nav-label">${escapeHtml(name)}</h2>`;
      const list = document.createElement("ul");
      list.className = "nav-list";
      for (const story of sectionStories) {
        const item = document.createElement("li");
        const button = document.createElement("button");
        button.type = "button";
        button.className = "nav-item";
        button.dataset.story = story.id;
        button.textContent = story.title;
        button.addEventListener("click", () => {
          selectStory(story.id, true);
          closeDrawers();
        });
        item.appendChild(button);
        list.appendChild(item);
      }
      section.appendChild(list);
      nav.appendChild(section);
    }
  }

  async function selectStory(id, writeHash) {
    const story = state.bootstrap.catalog.stories.find((item) => item.id === id);
    if (!story) return;

    state.storyId = id;
    state.values = {};
    for (const control of story.controls) {
      state.values[control.id] = control.default;
    }

    storyTitle.textContent = story.title;
    storyDesc.textContent = story.description;
    nav.querySelectorAll(".nav-item").forEach((button) => {
      button.classList.toggle("is-active", button.dataset.story === id);
    });
    renderControls(story);
    if (writeHash) {
      const next = `#/${id}`;
      if (location.hash !== next) {
        history.replaceState(null, "", next);
      }
    }
    await refresh();
  }

  function renderControls(story) {
    controlList.innerHTML = "";
    if (!story.controls.length) {
      const empty = document.createElement("p");
      empty.className = "controls-empty";
      empty.textContent = "This story has no controls.";
      controlList.appendChild(empty);
      return;
    }
    for (const control of story.controls) {
      const row = document.createElement("div");
      row.className = "control-row";
      const label = document.createElement("label");
      label.textContent = control.label;
      label.setAttribute("for", `ctrl-${control.id}`);
      row.appendChild(label);
      row.appendChild(buildControl(control));
      controlList.appendChild(row);
    }
  }

  function buildControl(control) {
    switch (control.kind) {
      case "select": {
        const select = document.createElement("select");
        select.id = `ctrl-${control.id}`;
        for (const option of control.options) {
          const item = document.createElement("option");
          item.value = option.value;
          item.textContent = option.label;
          select.appendChild(item);
        }
        select.value = String(state.values[control.id]);
        select.addEventListener("change", () => {
          state.values[control.id] = select.value;
          refresh();
        });
        return select;
      }
      case "number": {
        const input = document.createElement("input");
        input.id = `ctrl-${control.id}`;
        input.type = "number";
        if (control.min != null) input.min = String(control.min);
        if (control.max != null) input.max = String(control.max);
        input.value = String(state.values[control.id]);
        input.addEventListener("input", () => {
          const next = Number(input.value);
          state.values[control.id] = Number.isFinite(next) ? next : control.default;
          refresh();
        });
        return input;
      }
      case "boolean": {
        const wrap = document.createElement("label");
        wrap.className = "switch";
        const input = document.createElement("input");
        input.id = `ctrl-${control.id}`;
        input.type = "checkbox";
        input.checked = Boolean(state.values[control.id]);
        input.addEventListener("change", () => {
          state.values[control.id] = input.checked;
          refresh();
        });
        const track = document.createElement("span");
        wrap.append(input, track);
        return wrap;
      }
      case "text": {
        const input = document.createElement("input");
        input.id = `ctrl-${control.id}`;
        input.type = "text";
        input.value = String(state.values[control.id] ?? "");
        input.addEventListener("input", () => {
          state.values[control.id] = input.value;
          refresh();
        });
        return input;
      }
      default: {
        const _exhaustive = control.kind;
        void _exhaustive;
        return document.createTextNode("");
      }
    }
  }

  async function refresh() {
    showStatus("Updating preview…");
    try {
      const response = await fetch("/api/render", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ story: state.storyId, values: state.values }),
      });
      if (!response.ok) {
        throw new Error("render failed");
      }
      const payload = await response.json();
      hideStatus();
      codeBody.innerHTML = highlight(payload.code);
      sendRender(payload.html);
    } catch (error) {
      showStatus("Could not render this story.");
      console.error(error);
    }
  }

  function sendRender(html) {
    state.pending = html;
    if (!state.previewReady || !preview.contentWindow) return;
    preview.contentWindow.postMessage(
      { source: "schublade", type: "render", html, mode: state.mode },
      "*"
    );
  }

  function configurePreview() {
    if (!state.previewReady || !preview.contentWindow || !state.bootstrap) return;
    preview.contentWindow.postMessage(
      {
        source: "schublade",
        type: "configure",
        theme: {
          trigger: state.bootstrap.theme.trigger,
          key: state.bootstrap.theme.key,
          light: state.bootstrap.theme.light,
          dark: state.bootstrap.theme.dark,
        },
        a11y: state.bootstrap.a11y,
        mode: state.mode,
      },
      "*"
    );
  }

  function renderA11y(violations, skipped) {
    a11yList.innerHTML = "";
    if (skipped) {
      a11yToggle.textContent = "Accessibility · off";
      a11yToggle.classList.remove("has-issues", "is-ok");
      a11yEmpty.hidden = false;
      a11yEmpty.textContent = "Accessibility checks are disabled in schublade.toml.";
      return;
    }

    const count = violations.length;
    a11yToggle.textContent =
      count === 0 ? "Accessibility · 0 issues" : `Accessibility · ${count} ${count === 1 ? "issue" : "issues"}`;
    a11yToggle.classList.toggle("has-issues", count > 0);
    a11yToggle.classList.toggle("is-ok", count === 0);
    a11yEmpty.hidden = count > 0;
    a11yEmpty.textContent = "No issues with the current rules.";

    for (const item of violations) {
      const li = document.createElement("li");
      li.innerHTML = `${escapeHtml(item.label)} <span class="a11y-rule">${escapeHtml(item.rule)} · ${escapeHtml(item.target)}</span>`;
      a11yList.appendChild(li);
    }
  }

  function setViewport(name) {
    canvasFrame.dataset.viewport = name;
    document.querySelectorAll("[data-viewport]").forEach((button) => {
      const active = button.dataset.viewport === name;
      button.classList.toggle("is-active", active);
      button.setAttribute("aria-pressed", String(active));
    });
  }

  function toggleTheme() {
    state.mode = state.mode === "dark" ? "light" : "dark";
    localStorage.setItem("schublade:theme", state.mode);
    if (state.previewReady && preview.contentWindow) {
      preview.contentWindow.postMessage(
        { source: "schublade", type: "theme", mode: state.mode },
        "*"
      );
    }
  }

  function stepStory(delta) {
    const stories = state.bootstrap?.catalog.stories ?? [];
    if (!stories.length) return;
    const index = Math.max(0, stories.findIndex((story) => story.id === state.storyId));
    const next = stories[(index + delta + stories.length) % stories.length];
    selectStory(next.id, true);
  }

  async function copyUsage() {
    const text = codeBody.textContent || "";
    try {
      await navigator.clipboard.writeText(text);
      copyCode.setAttribute("aria-label", "Copied");
      setTimeout(() => copyCode.setAttribute("aria-label", "Copy usage"), 1200);
    } catch (_error) {
      copyCode.setAttribute("aria-label", "Copy failed");
    }
  }

  function toggleDrawer(panel) {
    const open = !panel.classList.contains("is-open");
    closeDrawers();
    panel.classList.toggle("is-open", open);
    scrim.hidden = !open;
  }

  function closeDrawers() {
    sidebar.classList.remove("is-open");
    controls.classList.remove("is-open");
    scrim.hidden = true;
  }

  function storyIdFromHash() {
    const raw = location.hash.replace(/^#\/?/, "");
    return raw || null;
  }

  function showStatus(text) {
    canvasStatus.hidden = false;
    canvasStatus.textContent = text;
  }

  function hideStatus() {
    canvasStatus.hidden = true;
  }

  function highlight(code) {
    return escapeHtml(code).replace(
      /(&lt;\/?[A-Za-z][\w.-]*)|(\s[\w:-]+=)|(&quot;.*?&quot;)/g,
      (match, tag, attr, str) => {
        if (tag) return `<span class="tok-tag">${tag}</span>`;
        if (attr) return `<span class="tok-attr">${attr}</span>`;
        return `<span class="tok-str">${str}</span>`;
      }
    );
  }

  function escapeHtml(value) {
    return String(value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }
})();
