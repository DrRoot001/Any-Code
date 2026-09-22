import { useCallback, useEffect, useRef, useState } from "react";
import { commands } from "../lib/tauri";
import { languageForPath } from "../lib/language";
import { loadMonaco, type Monaco } from "../lib/monaco";
import { useWorkbenchStore } from "../state/workbenchStore";

export default function EditorArea() {
  const tabs = useWorkbenchStore((s) => s.tabs);
  const activePath = useWorkbenchStore((s) => s.activePath);
  const setActivePath = useWorkbenchStore((s) => s.setActivePath);
  const closeTab = useWorkbenchStore((s) => s.closeTab);

  if (tabs.length === 0) {
    return (
      <div className="empty-state">
        <div>
          <strong>No file open</strong>Select a text file from Explorer to start editing.
        </div>
      </div>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div className="tabs" role="tablist" aria-label="Open files">
        {tabs.map((tab) => (
          <div
            className="tab"
            key={tab.path}
            role="presentation"
            aria-selected={activePath === tab.path}
          >
            <button
              className="tab-select"
              role="tab"
              aria-selected={activePath === tab.path}
              title={tab.path}
              onClick={() => setActivePath(tab.path)}
            >
              {tab.dirty && <span className="dirty-dot" aria-label="Unsaved" />}
              {tab.path.split("/").pop()}
            </button>
            <button
              className="tab-close"
              aria-label={`Close ${tab.path.split("/").pop()}`}
              onClick={() => {
                if (
                  !tab.dirty ||
                  window.confirm(`Discard unsaved changes to ${tab.path.split("/").pop()}?`)
                )
                  closeTab(tab.path);
              }}
            >
              ×
            </button>
          </div>
        ))}
      </div>
      <div style={{ flex: 1, minHeight: 0 }}>
        {tabs.map((tab) => (
          <div
            key={tab.path}
            style={{ height: "100%", display: tab.path === activePath ? "block" : "none" }}
          >
            <MonacoPane path={tab.path} initialContent={tab.savedContent} />
          </div>
        ))}
      </div>
    </div>
  );
}

function MonacoPane({ path, initialContent }: { path: string; initialContent: string }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<ReturnType<Monaco["editor"]["create"]> | null>(null);
  const markDirty = useWorkbenchStore((s) => s.markDirty);
  const markSaved = useWorkbenchStore((s) => s.markSaved);
  const savedContentRef = useRef(initialContent);
  const [ready, setReady] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  useEffect(() => {
    let disposed = false;
    let disposeEditor: (() => void) | null = null;

    loadMonaco()
      .then((monaco) => {
        if (disposed || !containerRef.current) return;
        const editor = monaco.editor.create(containerRef.current, {
          value: initialContent,
          language: languageForPath(path),
          automaticLayout: true,
          minimap: { enabled: false },
          fontFamily: "JetBrains Mono, ui-monospace, monospace",
          fontSize: 13,
        });
        editorRef.current = editor;
        setReady(true);

        const sub = editor.onDidChangeModelContent(() => {
          const dirty = editor.getValue() !== savedContentRef.current;
          markDirty(path, dirty);
        });

        disposeEditor = () => {
          sub.dispose();
          editor.dispose();
        };
      })
      .catch((error) => setSaveError(`Editor failed to load: ${String(error)}`));

    return () => {
      disposed = true;
      disposeEditor?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [path]);

  const save = useCallback(async () => {
    const editor = editorRef.current;
    if (!editor) return;
    const content = editor.getValue();
    try {
      // PRD §57: an agent must not silently overwrite — and must not be silently
      // overwritten. If the file moved on disk since this buffer was loaded (the agent
      // edited it), saying so beats quietly discarding one side of the change.
      const onDisk = await commands.readFile(path).catch(() => null);
      if (onDisk !== null && onDisk !== savedContentRef.current) {
        const overwrite = window.confirm(
          `${path} changed on disk since you opened it — the agent may have edited it.\n\n` +
            `Overwrite those changes with this editor's version?`,
        );
        if (!overwrite) {
          setSaveError(`Not saved. Close and reopen ${path} to load the version on disk.`);
          return;
        }
      }
      await commands.writeFile(path, content);
      savedContentRef.current = content;
      markSaved(path, content);
      setSaveError(null);
    } catch (error) {
      setSaveError(`Could not save ${path}: ${String(error)}`);
    }
  }, [path, markSaved]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "s") {
        e.preventDefault();
        save();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [save]);

  return (
    <div style={{ height: "100%", position: "relative" }}>
      {!ready && (
        <div
          style={{
            position: "absolute",
            inset: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: "var(--ac-text-muted)",
          }}
        >
          Loading editor…
        </div>
      )}
      <div ref={containerRef} style={{ height: "100%" }} />
      {saveError && (
        <div className="editor-message danger" role="alert">
          {saveError}
        </div>
      )}
    </div>
  );
}
