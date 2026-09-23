import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { commands } from "../lib/tauri";

function decodeBase64(data: string): Uint8Array {
  const binary = atob(data);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

export default function TerminalPanel() {
  const containerRef = useRef<HTMLDivElement>(null);
  const sessionIdRef = useRef<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;
    let disposed = false;
    let unlistenData: (() => void) | undefined;
    let unlistenExit: (() => void) | undefined;
    let disposeInput: (() => void) | undefined;
    let disposeResize: (() => void) | undefined;

    const term = new Terminal({
      fontFamily: "JetBrains Mono, ui-monospace, monospace",
      fontSize: 13,
      cursorBlink: true,
      theme: { background: "#00000000" },
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(containerRef.current);

    // Hiding the panel collapses the container to 0×0; fitting to that would ask the
    // PTY for a zero-column terminal, which some shells redraw very badly.
    const refit = () => {
      const box = containerRef.current;
      if (box && box.clientWidth > 0 && box.clientHeight > 0) fit.fit();
    };
    // Fit once synchronously so the shell is spawned at the size it will be drawn at,
    // rather than at xterm's 80×24 default.
    refit();
    const resizeObserver = new ResizeObserver(refit);
    resizeObserver.observe(containerRef.current);

    (async () => {
      const id = crypto.randomUUID();

      // Subscribe before the shell exists. Tauri events are not buffered, so a
      // listener registered after the spawn returns misses the first prompt.
      unlistenData = await listen<{ id: string; data: string }>(`terminal:data:${id}`, (event) => {
        term.write(decodeBase64(event.payload.data));
      });
      unlistenExit = await listen(`terminal:exit:${id}`, () => {
        term.write("\r\n[process exited]\r\n");
      });

      try {
        await commands.terminalSpawn(id, term.cols, term.rows);
      } catch (reason) {
        unlistenData?.();
        unlistenExit?.();
        throw reason;
      }
      if (disposed) {
        await commands.terminalKill(id);
        return;
      }
      sessionIdRef.current = id;

      const input = term.onData((data) => {
        commands
          .terminalWrite(id, data)
          .catch((reason) => setError(`Terminal input failed: ${String(reason)}`));
      });
      const resize = term.onResize(({ cols, rows }) => {
        commands
          .terminalResize(id, cols, rows)
          .catch((reason) => setError(`Terminal resize failed: ${String(reason)}`));
      });
      disposeInput = () => input.dispose();
      disposeResize = () => resize.dispose();
    })().catch((reason) => setError(`Terminal could not start: ${String(reason)}`));

    return () => {
      disposed = true;
      resizeObserver.disconnect();
      unlistenData?.();
      unlistenExit?.();
      disposeInput?.();
      disposeResize?.();
      if (sessionIdRef.current) {
        commands.terminalKill(sessionIdRef.current).catch(() => {});
      }
      term.dispose();
    };
  }, []);

  return (
    <div className="panel">
      {error && (
        <div className="notice notice--error" role="alert">
          {error}
        </div>
      )}
      <div ref={containerRef} className="terminal-body" />
    </div>
  );
}
