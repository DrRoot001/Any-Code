import { create } from "zustand";
import type { WorkspaceInfo } from "../lib/tauri";

export interface OpenTab {
  /** Workspace-relative path; doubles as the tab's identity. */
  path: string;
  /** Last-saved content, for dirty-checking against the live editor buffer. */
  savedContent: string;
  dirty: boolean;
}

export type BottomPanel = "terminal" | "chat" | "problems";
export type SidePanel = "explorer" | "git";

interface WorkbenchState {
  workspace: WorkspaceInfo | null;
  tabs: OpenTab[];
  activePath: string | null;
  sidePanel: SidePanel;
  bottomPanelOpen: boolean;
  bottomPanel: BottomPanel;
  commandPaletteOpen: boolean;
  diffPath: string | null;
  agentDockOpen: boolean;

  setWorkspace: (workspace: WorkspaceInfo | null) => void;
  openTab: (path: string, content: string) => void;
  closeTab: (path: string) => void;
  setActivePath: (path: string | null) => void;
  markDirty: (path: string, dirty: boolean) => void;
  markSaved: (path: string, content: string) => void;
  setSidePanel: (panel: SidePanel) => void;
  setBottomPanel: (panel: BottomPanel) => void;
  toggleBottomPanel: () => void;
  setCommandPaletteOpen: (open: boolean) => void;
  openDiff: (path: string) => void;
  closeDiff: () => void;
  toggleAgentDock: () => void;
}

export const useWorkbenchStore = create<WorkbenchState>((set) => ({
  workspace: null,
  tabs: [],
  activePath: null,
  sidePanel: "explorer",
  bottomPanelOpen: false,
  bottomPanel: "terminal",
  commandPaletteOpen: false,
  diffPath: null,
  agentDockOpen: true,

  setWorkspace: (workspace) =>
    set({ workspace, tabs: [], activePath: null, diffPath: null, sidePanel: "explorer" }),

  openTab: (path, content) =>
    set((state) => {
      if (state.tabs.some((t) => t.path === path)) {
        return { activePath: path };
      }
      return {
        tabs: [...state.tabs, { path, savedContent: content, dirty: false }],
        activePath: path,
      };
    }),

  closeTab: (path) =>
    set((state) => {
      const tabs = state.tabs.filter((t) => t.path !== path);
      const activePath = state.activePath === path ? (tabs.at(-1)?.path ?? null) : state.activePath;
      return { tabs, activePath };
    }),

  setActivePath: (activePath) => set({ activePath }),

  markDirty: (path, dirty) =>
    set((state) => ({
      tabs: state.tabs.map((t) => (t.path === path ? { ...t, dirty } : t)),
    })),

  markSaved: (path, content) =>
    set((state) => ({
      tabs: state.tabs.map((t) =>
        t.path === path ? { ...t, savedContent: content, dirty: false } : t,
      ),
    })),

  setSidePanel: (sidePanel) => set({ sidePanel }),
  setBottomPanel: (bottomPanel) => set({ bottomPanel, bottomPanelOpen: true }),
  toggleBottomPanel: () => set((state) => ({ bottomPanelOpen: !state.bottomPanelOpen })),
  setCommandPaletteOpen: (commandPaletteOpen) => set({ commandPaletteOpen }),
  openDiff: (diffPath) => set({ diffPath }),
  closeDiff: () => set({ diffPath: null }),
  toggleAgentDock: () => set((state) => ({ agentDockOpen: !state.agentDockOpen })),
}));
