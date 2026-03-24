import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface SessionInfo {
  id: string;
  title: string;
  cwd: string;
  createdAt: number;
}

interface PersistedSession {
  id: string;
  title: string;
  cwd: string;
  created_at: number;
  last_used_at: number;
}

interface SessionState {
  sessions: SessionInfo[];
  activeSessionId: string | null;

  addSession: (session: SessionInfo) => void;
  removeSession: (id: string) => void;
  setActiveSession: (id: string) => void;
  updateSessionTitle: (id: string, title: string) => void;
  updateSessionCwd: (id: string, cwd: string) => void;

  createNewSession: (cwd?: string, cols?: number, rows?: number) => Promise<string>;
  closeSession: (id: string) => Promise<void>;
  switchToNext: () => void;
  switchToPrevious: () => void;
  switchToIndex: (index: number) => void;

  /** Save current session state to disk via Rust backend. */
  saveSessionState: () => Promise<void>;
  /** Load persisted sessions from disk via Rust backend. */
  loadPersistedSessions: () => Promise<PersistedSession[]>;
  /** Restore a session at a given CWD (creates a new PTY session). */
  restoreSession: (title: string, cwd: string) => Promise<string>;
}

export const useSessionStore = create<SessionState>((set, get) => ({
  sessions: [],
  activeSessionId: null,

  addSession: (session) =>
    set((state) => ({
      sessions: [...state.sessions, session],
      activeSessionId: state.activeSessionId ?? session.id,
    })),

  removeSession: (id) =>
    set((state) => {
      const sessions = state.sessions.filter((s) => s.id !== id);
      let activeSessionId = state.activeSessionId;
      if (activeSessionId === id) {
        // Switch to next session, or previous, or null
        const idx = state.sessions.findIndex((s) => s.id === id);
        activeSessionId =
          sessions[Math.min(idx, sessions.length - 1)]?.id ?? null;
      }
      return { sessions, activeSessionId };
    }),

  setActiveSession: (id) => set({ activeSessionId: id }),

  updateSessionTitle: (id, title) =>
    set((state) => ({
      sessions: state.sessions.map((s) =>
        s.id === id ? { ...s, title } : s,
      ),
    })),

  updateSessionCwd: (id, cwd) =>
    set((state) => ({
      sessions: state.sessions.map((s) => (s.id === id ? { ...s, cwd } : s)),
    })),

  createNewSession: async (cwd?: string, cols?: number, rows?: number) => {
    const result = await invoke<{ id: string; cwd: string }>("create_session", {
      cwd: cwd ?? null,
      cols: cols ?? null,
      rows: rows ?? null,
    });
    get().addSession({
      id: result.id,
      title: `Terminal ${get().sessions.length + 1}`,
      cwd: result.cwd,
      createdAt: Date.now(),
    });
    get().setActiveSession(result.id);
    // Persist after creating
    get().saveSessionState();
    return result.id;
  },

  closeSession: async (id) => {
    try {
      await invoke("destroy_session", { sessionId: id });
    } catch {
      // Session may already be exited
    }
    get().removeSession(id);
    // Persist after closing
    get().saveSessionState();
  },

  switchToNext: () => {
    const { sessions, activeSessionId } = get();
    if (sessions.length <= 1) return;
    const idx = sessions.findIndex((s) => s.id === activeSessionId);
    const next = sessions[(idx + 1) % sessions.length];
    set({ activeSessionId: next.id });
  },

  switchToPrevious: () => {
    const { sessions, activeSessionId } = get();
    if (sessions.length <= 1) return;
    const idx = sessions.findIndex((s) => s.id === activeSessionId);
    const prev = sessions[(idx - 1 + sessions.length) % sessions.length];
    set({ activeSessionId: prev.id });
  },

  switchToIndex: (index) => {
    const { sessions } = get();
    if (index >= 0 && index < sessions.length) {
      set({ activeSessionId: sessions[index].id });
    }
  },

  saveSessionState: async () => {
    const { sessions } = get();
    const persisted: PersistedSession[] = sessions.map((s) => ({
      id: s.id,
      title: s.title,
      cwd: s.cwd,
      created_at: s.createdAt,
      last_used_at: Date.now(),
    }));
    try {
      await invoke("save_session_state", { sessions: persisted });
    } catch (err) {
      console.error("Failed to save session state:", err);
    }
  },

  loadPersistedSessions: async () => {
    try {
      const sessions = await invoke<PersistedSession[]>("load_session_state");
      return sessions;
    } catch (err) {
      console.error("Failed to load persisted sessions:", err);
      return [];
    }
  },

  restoreSession: async (title: string, cwd: string) => {
    const result = await invoke<{ id: string; cwd: string }>("create_session", { cwd });
    get().addSession({
      id: result.id,
      title,
      cwd: result.cwd,
      createdAt: Date.now(),
    });
    get().setActiveSession(result.id);
    // Persist after restoring
    get().saveSessionState();
    return result.id;
  },
}));
