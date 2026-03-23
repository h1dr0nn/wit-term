import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface SessionInfo {
  id: string;
  title: string;
  cwd: string;
  createdAt: number;
}

interface SessionState {
  sessions: SessionInfo[];
  activeSessionId: string | null;

  addSession: (session: SessionInfo) => void;
  removeSession: (id: string) => void;
  setActiveSession: (id: string) => void;
  updateSessionTitle: (id: string, title: string) => void;
  updateSessionCwd: (id: string, cwd: string) => void;

  createNewSession: (cwd?: string) => Promise<string>;
  closeSession: (id: string) => Promise<void>;
  switchToNext: () => void;
  switchToPrevious: () => void;
  switchToIndex: (index: number) => void;
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

  createNewSession: async (cwd?: string) => {
    const id = await invoke<string>("create_session", { cwd: cwd ?? null });
    get().addSession({
      id,
      title: `Terminal ${get().sessions.length + 1}`,
      cwd: cwd ?? "",
      createdAt: Date.now(),
    });
    get().setActiveSession(id);
    return id;
  },

  closeSession: async (id) => {
    try {
      await invoke("destroy_session", { sessionId: id });
    } catch {
      // Session may already be exited
    }
    get().removeSession(id);
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
}));
