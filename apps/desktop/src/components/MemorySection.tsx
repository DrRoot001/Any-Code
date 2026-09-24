import { save as saveFileDialog } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useState } from "react";
import { scopeLabel, sourceLabel } from "../lib/memory";
import {
  memoryCommands,
  type InstructionFile,
  type Memory,
  type MemoryScope,
} from "../lib/tauri";
import { useWorkbenchStore } from "../state/workbenchStore";

/**
 * Memory (ADR 0004 "Memory" / PRD §38): the user's own words, global or for this
 * workspace, all visible, editable, deletable and exportable — and instruction files a
 * cloned repository brought with it, which are untrusted content until the user adopts
 * one on purpose.
 */
export default function MemorySection() {
  const workspace = useWorkbenchStore((s) => s.workspace);

  const [memories, setMemories] = useState<Memory[] | null>(null);
  const [memoriesError, setMemoriesError] = useState<string | null>(null);
  const [files, setFiles] = useState<InstructionFile[] | null>(null);
  const [filesError, setFilesError] = useState<string | null>(null);

  const [newScope, setNewScope] = useState<MemoryScope>("global");
  const [newContent, setNewContent] = useState("");
  const [addError, setAddError] = useState<string | null>(null);
  const [exportError, setExportError] = useState<string | null>(null);
  const [busy, setBusy] = useState<string | null>(null);

  const refreshMemories = useCallback(() => {
    memoryCommands
      .listMemories()
      .then((list) => {
        setMemories(list);
        setMemoriesError(null);
      })
      .catch((reason) => setMemoriesError(String(reason)));
  }, []);

  const refreshFiles = useCallback(() => {
    if (!workspace) {
      setFiles([]);
      return;
    }
    memoryCommands
      .repositoryInstructions()
      .then((list) => {
        setFiles(list);
        setFilesError(null);
      })
      .catch((reason) => setFilesError(String(reason)));
  }, [workspace]);

  useEffect(refreshMemories, [refreshMemories]);
  useEffect(refreshFiles, [refreshFiles]);

  const addMemory = async () => {
    const content = newContent.trim();
    if (!content) return;
    setBusy("add");
    setAddError(null);
    try {
      await memoryCommands.addMemory(newScope, content);
      setNewContent("");
      refreshMemories();
    } catch (reason) {
      setAddError(String(reason));
    } finally {
      setBusy(null);
    }
  };

  // Adopting turns repository text into standing instructions, so the user reads the
  // exact text first, and only that text is adopted.
  const [preview, setPreview] = useState<{ path: string; content: string } | null>(null);

  const review = async (path: string) => {
    setBusy(path);
    setFilesError(null);
    try {
      setPreview({ path, content: await memoryCommands.previewInstruction(path) });
    } catch (reason) {
      setFilesError(String(reason));
    } finally {
      setBusy(null);
    }
  };

  const adopt = async () => {
    if (!preview) return;
    setBusy(preview.path);
    try {
      await memoryCommands.adoptInstruction(preview.path, preview.content);
      setPreview(null);
      refreshMemories();
      refreshFiles();
    } catch (reason) {
      setFilesError(String(reason));
    } finally {
      setBusy(null);
    }
  };

  const exportMemories = async () => {
    setExportError(null);
    try {
      const path = await saveFileDialog({
        title: "Export memories",
        defaultPath: "any-code-memories.json",
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!path) return;
      await memoryCommands.exportMemories(path);
    } catch (reason) {
      setExportError(String(reason));
    }
  };

  return (
    <div className="memory-section">
      {memoriesError && (
        <p className="danger" role="alert">
          {memoriesError}
        </p>
      )}
      {!memoriesError && memories === null && <p className="muted">Loading memories…</p>}
      {!memoriesError && memories && memories.length === 0 && (
        <p className="muted">No memories yet.</p>
      )}
      {!memoriesError && memories && memories.length > 0 && (
        <ul className="memory-list">
          {memories.map((memory) => (
            <MemoryRow key={memory.id} memory={memory} onChanged={refreshMemories} />
          ))}
        </ul>
      )}

      <form
        className="memory-add"
        onSubmit={(e) => {
          e.preventDefault();
          addMemory();
        }}
      >
        <div className="memory-scope-choice" role="radiogroup" aria-label="Memory scope">
          <label className="setting-option">
            <input
              type="radio"
              name="memory-scope"
              checked={newScope === "global"}
              onChange={() => setNewScope("global")}
            />{" "}
            Global
          </label>
          <label className="setting-option">
            <input
              type="radio"
              name="memory-scope"
              checked={newScope === "workspace"}
              disabled={!workspace}
              onChange={() => setNewScope("workspace")}
            />{" "}
            This workspace
          </label>
        </div>
        {!workspace && (
          <p className="muted">Open a workspace to add a memory scoped to it.</p>
        )}
        <textarea
          className="memory-textarea"
          value={newContent}
          onChange={(e) => setNewContent(e.target.value)}
          placeholder="Write something for the agent to remember…"
          aria-label="New memory"
          rows={3}
        />
        {addError && (
          <p className="danger" role="alert">
            {addError}
          </p>
        )}
        <button className="button" type="submit" disabled={!newContent.trim() || busy === "add"}>
          Add memory
        </button>
      </form>

      <div className="memory-export">
        <button className="button" onClick={exportMemories}>
          Export memories…
        </button>
        {exportError && (
          <p className="danger" role="alert">
            {exportError}
          </p>
        )}
      </div>

      <h4>Instructions in this repository</h4>
      <p className="muted">
        These files are repository content, not your instructions — the agent does not follow
        them unless you adopt one, which copies its text into this workspace's memory.
      </p>
      {!workspace && <p className="muted">Open a workspace to see its instruction files.</p>}
      {workspace && filesError && (
        <p className="danger" role="alert">
          {filesError}
        </p>
      )}
      {workspace && !filesError && files === null && (
        <p className="muted">Loading instruction files…</p>
      )}
      {workspace && !filesError && files && files.length === 0 && (
        <p className="muted">No instruction files found in this repository.</p>
      )}
      {workspace && !filesError && files && files.length > 0 && (
        <ul className="instruction-list">
          {files.map((file) => (
            <li key={file.path} className="instruction-row">
              <code>{file.path}</code>
              <span className="muted">{file.bytes.toLocaleString()} bytes</span>
              {file.adopted ? (
                <span className="muted">Adopted</span>
              ) : (
                <button
                  className="button"
                  onClick={() => review(file.path)}
                  disabled={busy === file.path || preview?.path === file.path}
                >
                  Review to adopt
                </button>
              )}
            </li>
          ))}
        </ul>
      )}
      {preview && (
        <section className="instruction-preview" aria-label={`Contents of ${preview.path}`}>
          <p>
            Adopting <code>{preview.path}</code> makes this text your instructions to the agent
            on every task in this workspace:
          </p>
          {/* Repository text: a plain text child, never rendered as markup. */}
          <pre className="context-passage">
            <code>{preview.content}</code>
          </pre>
          <div className="instruction-preview-actions">
            <button className="button button--primary" onClick={adopt} disabled={busy === preview.path}>
              Adopt this text
            </button>
            <button className="button" onClick={() => setPreview(null)}>
              Cancel
            </button>
          </div>
        </section>
      )}
    </div>
  );
}

function MemoryRow({ memory, onChanged }: { memory: Memory; onChanged: () => void }) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(memory.content);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const save = async () => {
    const content = draft.trim();
    if (!content) return;
    setBusy(true);
    setError(null);
    try {
      await memoryCommands.updateMemory(memory.id, content);
      setEditing(false);
      onChanged();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  };

  const remove = async () => {
    if (!window.confirm("Delete this memory? This cannot be undone.")) return;
    setBusy(true);
    setError(null);
    try {
      await memoryCommands.deleteMemory(memory.id);
      onChanged();
    } catch (reason) {
      setError(String(reason));
      setBusy(false);
    }
  };

  const source = sourceLabel(memory.source);

  return (
    <li className="memory-row">
      <div className="memory-row-heading">
        <span className="memory-scope">{scopeLabel(memory.scope)}</span>
        {source && <span className="muted">{source}</span>}
      </div>
      {editing ? (
        <textarea
          className="memory-textarea"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          aria-label="Edit memory"
          rows={3}
        />
      ) : (
        <p>{memory.content}</p>
      )}
      {error && (
        <p className="danger" role="alert">
          {error}
        </p>
      )}
      <div className="memory-row-actions">
        {editing ? (
          <>
            <button className="button" onClick={save} disabled={busy || !draft.trim()}>
              Save
            </button>
            <button
              className="button"
              onClick={() => {
                setDraft(memory.content);
                setEditing(false);
                setError(null);
              }}
              disabled={busy}
            >
              Cancel
            </button>
          </>
        ) : (
          <>
            <button className="button" onClick={() => setEditing(true)}>
              Edit
            </button>
            <button className="button" onClick={remove} disabled={busy}>
              Delete
            </button>
          </>
        )}
      </div>
    </li>
  );
}
