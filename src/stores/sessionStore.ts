import { create } from "zustand";

export interface SessionInfo {
  id: string;
  title: string;
  cwd: string;
}

interface SessionState {
  sessions: Map<string, SessionInfo>;
  activeSessionId: string | null;
  setActiveSession: (id: string) => void;
}

export const useSessionStore = create<SessionState>((set) => ({
  sessions: new Map(),
  activeSessionId: null,
  setActiveSession: (id) => set({ activeSessionId: id }),
}));
