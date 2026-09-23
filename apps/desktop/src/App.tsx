import { open as openFolderDialog } from "@tauri-apps/plugin-dialog";
import { applyTheme, type ThemeName } from "@anycode/design-tokens";
import { useCallback, useEffect, useMemo, useState } from "react";
import logo from "../../../assets/brand/any-code-mark.png";
import AgentPanel from "./components/AgentPanel";
import ChatPanel from "./components/ChatPanel";
import CommandPalette, { type Command } from "./components/CommandPalette";
import DiffPane from "./components/DiffPane";
import EditorArea from "./components/EditorArea";
import Explorer from "./components/Explorer";
import GitPanel from "./components/GitPanel";
import { Icon } from "./components/Icons";
import SettingsPanel from "./components/SettingsPanel";
import StatusBar from "./components/StatusBar";
import TerminalPanel from "./components/TerminalPanel";
import WelcomeScreen from "./components/WelcomeScreen";
import { commands } from "./lib/tauri";
import { useWorkbenchStore } from "./state/workbenchStore";

const THEMES: ThemeName[] = ["system", "light", "dark", "high-contrast"];
export default function App() {
  const [theme, setTheme] = useState<ThemeName | null>(null),
    [settingsOpen, setSettingsOpen] = useState(false),
    [notice, setNotice] = useState<string | null>(null);
  const workspace = useWorkbenchStore((s) => s.workspace),
    setWorkspace = useWorkbenchStore((s) => s.setWorkspace),
    sidePanel = useWorkbenchStore((s) => s.sidePanel),
    setSidePanel = useWorkbenchStore((s) => s.setSidePanel),
    bottomPanelOpen = useWorkbenchStore((s) => s.bottomPanelOpen),
    toggleBottomPanel = useWorkbenchStore((s) => s.toggleBottomPanel),
    bottomPanel = useWorkbenchStore((s) => s.bottomPanel),
    setBottomPanel = useWorkbenchStore((s) => s.setBottomPanel),
    diffPath = useWorkbenchStore((s) => s.diffPath),
    agentDockOpen = useWorkbenchStore((s) => s.agentDockOpen),
    toggleAgentDock = useWorkbenchStore((s) => s.toggleAgentDock),
    paletteOpen = useWorkbenchStore((s) => s.commandPaletteOpen),
    setPaletteOpen = useWorkbenchStore((s) => s.setCommandPaletteOpen);
  // Mounted on first use and never unmounted again: spawning a shell before the user
  // asks for one would also spawn it before a workspace exists, which fails.
  const terminalVisible = bottomPanelOpen && bottomPanel === "terminal";
  const [terminalMounted, setTerminalMounted] = useState(false);
  useEffect(() => {
    if (terminalVisible) setTerminalMounted(true);
  }, [terminalVisible]);
  useEffect(() => {
    commands
      .getTheme()
      .then((value) => {
        const loaded = THEMES.includes(value as ThemeName) ? (value as ThemeName) : "system";
        setTheme(loaded);
        applyTheme(loaded);
      })
      .catch(() => {
        setTheme("system");
        applyTheme("system");
        setNotice("Theme preference could not be loaded. System theme is in use.");
      });
    commands
      .getLastWorkspace()
      .then((info) => info && setWorkspace(info))
      .catch(() => setNotice("The previous workspace could not be reopened."));
  }, [setWorkspace]);
  const openFolder = useCallback(async () => {
    try {
      const selected = await openFolderDialog({
        directory: true,
        multiple: false,
        title: "Open repository",
      });
      if (typeof selected === "string") setWorkspace(await commands.openWorkspace(selected));
    } catch (error) {
      setNotice(`Could not open the repository: ${String(error)}`);
    }
  }, [setWorkspace]);
  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key.toLowerCase() === "p") {
        event.preventDefault();
        setPaletteOpen(!paletteOpen);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [paletteOpen, setPaletteOpen]);
  const paletteCommands = useMemo<Command[]>(
    () => [
      { id: "open-folder", label: "Workspace: Open Repository…", run: openFolder },
      { id: "toggle-terminal", label: "View: Toggle Terminal", run: toggleBottomPanel },
      { id: "show-explorer", label: "View: Show Explorer", run: () => setSidePanel("explorer") },
      { id: "show-git", label: "View: Show Source Control", run: () => setSidePanel("git") },
      { id: "show-chat", label: "View: Show Chat", run: () => setBottomPanel("chat") },
      { id: "toggle-agent", label: "View: Toggle Agent", run: toggleAgentDock },
      {
        id: "open-settings",
        label: "Preferences: Open Settings",
        run: () => setSettingsOpen(true),
      },
    ],
    [openFolder, setSidePanel, toggleBottomPanel, setBottomPanel, toggleAgentDock],
  );
  if (theme === null)
    return (
      <main className="loading-screen" aria-live="polite">
        Starting Any Code…
      </main>
    );
  if (!workspace)
    return (
      <>
        <WelcomeScreen onOpenFolder={openFolder} />
        {notice && <Notice text={notice} onClose={() => setNotice(null)} />}
        <CommandPalette commands={paletteCommands} />
      </>
    );
  return (
    <div className="app">
      <header className="titlebar">
        <div className="brand">
          <img src={logo} alt="" />
          <span>Any Code</span>
          <span className="brand-credit">by Heptagram AI</span>
        </div>
        <div className="workspace-title" title={workspace.path}>
          {workspace.name}
        </div>
        <div className="toolbar">
          <button
            className="button"
            onClick={toggleAgentDock}
            aria-pressed={agentDockOpen}
            title="Toggle the agent dock"
          >
            <Icon name="chat" />
            <span className="button-label">Agent</span>
          </button>
          <button className="button" onClick={() => setPaletteOpen(true)}>
            <Icon name="command" />
            <span className="button-label">Commands</span>
            <span className="shortcut">Ctrl/⌘⇧P</span>
          </button>
          <button
            className="icon-button"
            onClick={openFolder}
            title="Open another repository"
            aria-label="Open another repository"
          >
            <Icon name="folder" />
          </button>
        </div>
      </header>
      {notice && <Notice text={notice} onClose={() => setNotice(null)} />}
      <div className="workbench">
        <nav className="activity-bar" aria-label="Primary views">
          <Activity
            label="Explorer"
            active={sidePanel === "explorer"}
            onClick={() => setSidePanel("explorer")}
            icon="files"
          />
          <Activity
            label="Source Control"
            active={sidePanel === "git"}
            onClick={() => setSidePanel("git")}
            icon="branch"
          />
        </nav>
        <aside
          className="sidebar"
          aria-label={sidePanel === "explorer" ? "Explorer" : "Source control"}
        >
          {sidePanel === "explorer" ? <Explorer /> : <GitPanel />}
        </aside>
        <main className="main-area">
          <div className="editor-region">
            {diffPath ? <DiffPane path={diffPath} key={diffPath} /> : <EditorArea />}
          </div>
          {/* Kept mounted, never conditionally rendered: unmounting TerminalPanel kills
              its shell, so closing the panel or switching to Chat would discard whatever
              was running in it. */}
          <section
            className="bottom-panel"
            aria-label={bottomPanel === "chat" ? "Chat" : "Terminal"}
            hidden={!bottomPanelOpen}
          >
              <div className="panel-header">
                <div className="bottom-panel-tabs" role="tablist" aria-label="Bottom panel">
                  <button
                    role="tab"
                    aria-selected={bottomPanel === "terminal"}
                    className={
                      bottomPanel === "terminal" ? "bottom-tab bottom-tab--active" : "bottom-tab"
                    }
                    onClick={() => setBottomPanel("terminal")}
                  >
                    Terminal
                  </button>
                  <button
                    role="tab"
                    aria-selected={bottomPanel === "chat"}
                    className={
                      bottomPanel === "chat" ? "bottom-tab bottom-tab--active" : "bottom-tab"
                    }
                    onClick={() => setBottomPanel("chat")}
                  >
                    Chat
                  </button>
                </div>
                <button
                  className="icon-button"
                  onClick={toggleBottomPanel}
                  aria-label="Close panel"
                >
                  <Icon name="close" size={14} />
                </button>
              </div>
              <div className="bottom-panel-body" hidden={bottomPanel !== "terminal"}>
                {terminalMounted && <TerminalPanel />}
              </div>
              <div className="bottom-panel-body" hidden={bottomPanel !== "chat"}>
                <ChatPanel />
              </div>
            </section>
        </main>
        <aside className="agent-dock" aria-label="Agent" hidden={!agentDockOpen}>
          <div className="panel-header">
            <span>Agent</span>
            <button className="icon-button" onClick={toggleAgentDock} aria-label="Close agent">
              <Icon name="close" size={14} />
            </button>
          </div>
          <div className="agent-dock-body">
            <AgentPanel />
          </div>
        </aside>
      </div>
      <StatusBar onOpenSettings={() => setSettingsOpen(true)} />
      <CommandPalette commands={paletteCommands} />
      {settingsOpen && (
        <SettingsPanel
          theme={theme}
          onThemeChange={setTheme}
          onClose={() => setSettingsOpen(false)}
        />
      )}
    </div>
  );
}
function Activity({
  label,
  active,
  onClick,
  icon,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
  icon: "files" | "branch";
}) {
  return (
    <button
      className="activity-button"
      title={label}
      aria-label={label}
      aria-pressed={active}
      onClick={onClick}
    >
      <Icon name={icon} size={19} />
    </button>
  );
}
function Notice({ text, onClose }: { text: string; onClose: () => void }) {
  return (
    <div className="notice notice--error" role="alert">
      <span>{text}</span>
      <button className="icon-button" onClick={onClose} aria-label="Dismiss message">
        <Icon name="close" />
      </button>
    </div>
  );
}
