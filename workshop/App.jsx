import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  IconAdjustmentsHorizontal,
  IconChevronLeft,
  IconChevronRight,
  IconMenu2,
} from "./icons.js";
import { Controls } from "./Controls.jsx";
import { IconButton } from "./IconButton.jsx";
import { Inspector } from "./Inspector.jsx";
import { Sidebar } from "./Sidebar.jsx";

export function App({ initialBootstrap, staticMode }) {
  const [bootstrap, setBootstrap] = useState(initialBootstrap);
  const [storyId, setStoryId] = useState(null);
  const [values, setValues] = useState({});
  const [mode, setMode] = useState(() => localStorage.getItem("schublade:theme") || "light");
  const [viewport, setViewport] = useState("desktop");
  const [tab, setTab] = useState("code");
  const [code, setCode] = useState("");
  const [status, setStatus] = useState(initialBootstrap ? "Loading story…" : "Loading catalog…");
  const [error, setError] = useState(null);
  const [copied, setCopied] = useState(false);
  const [navOpen, setNavOpen] = useState(false);
  const [controlsOpen, setControlsOpen] = useState(false);
  const [a11y, setA11y] = useState({ violations: [], skipped: false });
  const previewRef = useRef(null);
  const previewReady = useRef(false);
  const pendingRender = useRef(null);
  const isStatic = useRef(staticMode || Boolean(initialBootstrap?.static));
  const storyIdRef = useRef(storyId);
  const valuesRef = useRef(values);
  storyIdRef.current = storyId;
  valuesRef.current = values;

  const stories = bootstrap?.catalog.stories ?? [];
  const story = useMemo(
    () => stories.find((item) => item.id === storyId) ?? null,
    [stories, storyId]
  );
  const catalogName = bootstrap?.brand?.name || bootstrap?.catalog.name || "";
  const logoUrl = bootstrap?.brand?.logo || null;

  const closeDrawers = useCallback(() => {
    setNavOpen(false);
    setControlsOpen(false);
  }, []);

  const configurePreview = useCallback(
    (nextBootstrap) => {
      const frame = previewRef.current;
      if (!previewReady.current || !frame?.contentWindow || !nextBootstrap) return;
      frame.contentWindow.postMessage(
        {
          source: "schublade",
          type: "configure",
          theme: {
            trigger: nextBootstrap.theme.trigger,
            key: nextBootstrap.theme.key,
            light: nextBootstrap.theme.light,
            dark: nextBootstrap.theme.dark,
          },
          a11y: nextBootstrap.a11y,
          mode,
        },
        "*"
      );
    },
    [mode]
  );

  const sendRender = useCallback(
    (payload) => {
      pendingRender.current = payload;
      const frame = previewRef.current;
      if (!previewReady.current || !frame?.contentWindow) return;
      frame.contentWindow.postMessage(
        {
          source: "schublade",
          type: "render",
          html: payload.html,
          react: payload.react ?? null,
          mode,
        },
        "*"
      );
    },
    [mode]
  );

  const renderCurrent = useCallback(
    async (nextBootstrap, id, nextValues) => {
      if (isStatic.current && typeof window.SchubladeRender !== "undefined") {
        const current = nextBootstrap.catalog.stories.find((item) => item.id === id);
        if (!current) throw new Error("unknown story");
        const rendered = window.SchubladeRender.renderStory(current, nextValues);
        return {
          html: rendered.html,
          code: rendered.code,
          react: rendered.react ?? null,
        };
      }
      const response = await fetch("/api/render", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ story: id, values: nextValues }),
      });
      if (!response.ok) throw new Error("render failed");
      return response.json();
    },
    []
  );

  const refresh = useCallback(
    async (nextBootstrap, id, nextValues) => {
      setStatus("Updating preview…");
      try {
        const payload = await renderCurrent(nextBootstrap, id, nextValues);
        setCode(payload.code || "");
        setStatus(null);
        setError(null);
        sendRender(payload);
      } catch (cause) {
        setStatus("Could not render this story.");
        setError(cause);
        console.error(cause);
      }
    },
    [renderCurrent, sendRender]
  );

  const defaultsFor = useCallback((nextStory) => {
    const next = {};
    for (const control of nextStory.controls) {
      next[control.id] = control.default;
    }
    return next;
  }, []);

  const selectStory = useCallback(
    (id, writeHash, nextBootstrap = bootstrap) => {
      if (!nextBootstrap) return;
      const nextStory = nextBootstrap.catalog.stories.find((item) => item.id === id);
      if (!nextStory) return;
      const nextValues = defaultsFor(nextStory);
      setStoryId(id);
      setValues(nextValues);
      if (writeHash) {
        const hash = `#/${id}`;
        if (location.hash !== hash) history.replaceState(null, "", hash);
      }
      refresh(nextBootstrap, id, nextValues);
    },
    [bootstrap, defaultsFor, refresh]
  );

  const showEmptyCatalog = useCallback((nextBootstrap) => {
    setStoryId(null);
    setValues({});
    setCode("");
    setStatus("This catalog has no stories yet.");
    document.title = `${nextBootstrap.catalog.name} · Schublade`;
  }, []);

  const applyBootstrap = useCallback(
    (next, keepSelection) => {
      isStatic.current = Boolean(next.static);
      setBootstrap(next);
      document.title = `${next.catalog.name} · Schublade`;
      const nextStories = next.catalog.stories;
      if (!nextStories.length) {
        showEmptyCatalog(next);
        return;
      }
      const requested = storyIdFromHash();
      const fallback =
        nextStories.find((item) => item.id === "avatar-group")?.id || nextStories[0].id;
      const previousId = storyIdRef.current;
      const previousValues = valuesRef.current;
      const nextId = keepSelection
        ? nextStories.find((item) => item.id === previousId)?.id ||
          requested ||
          fallback
        : requested || fallback;
      const nextStory = nextStories.find((item) => item.id === nextId);
      const nextValues = {};
      for (const control of nextStory.controls) {
        const keep =
          keepSelection &&
          previousId === nextStory.id &&
          Object.prototype.hasOwnProperty.call(previousValues, control.id);
        nextValues[control.id] = keep ? previousValues[control.id] : control.default;
      }
      setStoryId(nextStory.id);
      setValues(nextValues);
      const hash = `#/${nextStory.id}`;
      if (location.hash !== hash) history.replaceState(null, "", hash);
      configurePreview(next);
      refresh(next, nextStory.id, nextValues);
    },
    [configurePreview, refresh, showEmptyCatalog]
  );

  useEffect(() => {
    let cancelled = false;
    if (initialBootstrap) {
      applyBootstrap(initialBootstrap, false);
      return;
    }
    loadBootstrap()
      .then((next) => {
        if (cancelled) return;
        isStatic.current = next.staticMode;
        applyBootstrap(next.bootstrap, false);
      })
      .catch((cause) => {
        if (cancelled) return;
        setStatus("Could not load the workshop catalog.");
        setError(cause);
        console.error(cause);
      });
    return () => {
      cancelled = true;
    };
    // Boot once from the injected payload or /api/bootstrap.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!bootstrap || isStatic.current) return undefined;
    if (typeof EventSource === "undefined") {
      let generation = null;
      let timer = 0;
      const poll = async () => {
        try {
          const response = await fetch("/api/generation");
          if (response.ok) {
            const payload = await response.json();
            if (generation !== null && payload.generation !== generation) {
              const next = await loadBootstrap();
              isStatic.current = next.staticMode;
              applyBootstrap(next.bootstrap, true);
            }
            generation = payload.generation;
          }
        } catch (_cause) {
          /* server went away; retry */
        }
        timer = window.setTimeout(poll, 400);
      };
      poll();
      return () => window.clearTimeout(timer);
    }
    const events = new EventSource("/api/events");
    events.addEventListener("reload", () => {
      loadBootstrap()
        .then((next) => {
          isStatic.current = next.staticMode;
          applyBootstrap(next.bootstrap, true);
        })
        .catch((cause) => console.error(cause));
    });
    return () => events.close();
  }, [applyBootstrap, bootstrap]);

  useEffect(() => {
    const onHash = () => {
      if (!bootstrap || !stories.length) return;
      selectStory(storyIdFromHash() || stories[0].id, false);
    };
    window.addEventListener("hashchange", onHash);
    return () => window.removeEventListener("hashchange", onHash);
  }, [bootstrap, selectStory, stories]);

  useEffect(() => {
    const onMessage = (event) => {
      const message = event.data;
      if (!message || message.source !== "schublade-preview") return;
      switch (message.type) {
        case "ready":
          previewReady.current = true;
          configurePreview(bootstrap);
          if (pendingRender.current) sendRender(pendingRender.current);
          break;
        case "a11y":
          setA11y({
            violations: message.violations || [],
            skipped: Boolean(message.skipped),
          });
          break;
        default: {
          const _never = message.type;
          void _never;
        }
      }
    };
    window.addEventListener("message", onMessage);
    return () => window.removeEventListener("message", onMessage);
  }, [bootstrap, configurePreview, sendRender]);

  useEffect(() => {
    localStorage.setItem("schublade:theme", mode);
    const frame = previewRef.current;
    if (previewReady.current && frame?.contentWindow) {
      frame.contentWindow.postMessage(
        { source: "schublade", type: "theme", mode },
        "*"
      );
    }
  }, [mode]);

  function stepStory(delta) {
    if (!stories.length) return;
    const index = Math.max(
      0,
      stories.findIndex((item) => item.id === storyId)
    );
    const next = stories[(index + delta + stories.length) % stories.length];
    selectStory(next.id, true);
  }

  async function copyUsage() {
    try {
      await navigator.clipboard.writeText(code || "");
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1200);
    } catch (_cause) {
      setCopied(false);
    }
  }

  function updateValue(id, next) {
    const merged = { ...values, [id]: next };
    setValues(merged);
    if (bootstrap && storyId) refresh(bootstrap, storyId, merged);
  }

  const emptyDescription =
    "This catalog is empty. Add *.stories.jsx / *.stories.js files or [[stories]] in catalog.toml. The workshop reloads when those files change.";

  return (
    <div className="app">
      <Sidebar
        catalogName={catalogName}
        logoUrl={logoUrl}
        stories={stories}
        storyId={storyId}
        onSelect={(id) => selectStory(id, true)}
        open={navOpen}
        onClose={closeDrawers}
      />
      <div className="stage">
        <header className="stage-head">
          <div className="stage-nav">
            <IconButton
              label="Previous story"
              disabled={!stories.length}
              onClick={() => stepStory(-1)}
            >
              <IconChevronLeft size={18} stroke={1.6} />
            </IconButton>
            <IconButton
              label="Next story"
              disabled={!stories.length}
              onClick={() => stepStory(1)}
            >
              <IconChevronRight size={18} stroke={1.6} />
            </IconButton>
          </div>
          <h1 className="story-title">{story ? story.title : stories.length ? "Loading" : "No stories"}</h1>
          <div className="stage-actions">
            <button
              type="button"
              className="text-btn mobile-only"
              onClick={() => {
                setControlsOpen(false);
                setNavOpen((open) => !open);
              }}
            >
              <IconMenu2 size={16} stroke={1.6} />
              Stories
            </button>
            <button
              type="button"
              className="text-btn mobile-only"
              onClick={() => {
                setNavOpen(false);
                setControlsOpen((open) => !open);
              }}
            >
              <IconAdjustmentsHorizontal size={16} stroke={1.6} />
              Controls
            </button>
          </div>
        </header>
        <div className="canvas-wrap">
          <div className="canvas-frame" data-viewport={viewport} id="canvas-frame">
            <iframe
              ref={previewRef}
              id="preview"
              title="Isolated component preview"
              sandbox="allow-scripts"
              src={isStatic.current ? "./preview.html" : "/preview"}
            />
            {status ? (
              <p className="canvas-status" id="canvas-status">
                {status}
              </p>
            ) : null}
          </div>
        </div>
        {!stories.length && !error ? (
          <p className="stage-empty">{emptyDescription}</p>
        ) : null}
        <Inspector
          tab={tab}
          onTabChange={setTab}
          code={code}
          copied={copied}
          onCopy={copyUsage}
          viewport={viewport}
          onViewport={setViewport}
          mode={mode}
          onMode={setMode}
          a11y={a11y}
        />
      </div>
      <Controls
        story={story}
        values={values}
        onChange={updateValue}
        open={controlsOpen}
      />
      <div
        className="scrim"
        hidden={!navOpen && !controlsOpen}
        onClick={closeDrawers}
      />
    </div>
  );
}

function storyIdFromHash() {
  const raw = location.hash.replace(/^#\/?/, "");
  return raw || null;
}

export async function loadBootstrap() {
  const injected = document.getElementById("schublade-bootstrap");
  if (injected) {
    const text = injected.textContent.trim();
    if (text) {
      const bootstrap = JSON.parse(text);
      return { bootstrap, staticMode: Boolean(bootstrap.static) };
    }
  }
  try {
    const response = await fetch("/api/bootstrap");
    if (response.ok) {
      return { bootstrap: await response.json(), staticMode: false };
    }
  } catch (_error) {
    /* static host or file:// */
  }
  const response = await fetch("./bootstrap.json");
  if (!response.ok) throw new Error("bootstrap failed");
  const bootstrap = await response.json();
  return { bootstrap, staticMode: true };
}
