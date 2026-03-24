import { create } from "zustand";

export interface AgentIdentity {
  name: string;
  kind: string;
  pid: number;
  detectedAt: number;
}

export interface AgentTimelineEvent {
  id: string;
  eventType: string;
  data: Record<string, unknown>;
  timestamp: number;
}

export interface AgentFileChange {
  path: string;
  action: "created" | "modified" | "deleted";
  timestamp: number;
}

export interface AgentSessionInfo {
  identity: AgentIdentity;
  events: AgentTimelineEvent[];
  fileChanges: AgentFileChange[];
  totalInputTokens: number;
  totalOutputTokens: number;
  totalCost: number;
  currentModel: string;
  currentFile: string;
  isThinking: boolean;
  isEnded: boolean;
}

interface AgentState {
  sessions: Record<string, AgentSessionInfo>;
  sidebarVisible: boolean;
  activeTab: "activity" | "files";

  setAgent: (sessionId: string, identity: AgentIdentity) => void;
  endAgent: (sessionId: string) => void;
  addEvent: (sessionId: string, event: AgentTimelineEvent) => void;
  addFileChange: (sessionId: string, change: AgentFileChange) => void;
  updateTokens: (sessionId: string, input: number, output: number) => void;
  updateCost: (sessionId: string, cost: number) => void;
  updateModel: (sessionId: string, model: string) => void;
  updateFile: (sessionId: string, file: string) => void;
  setThinking: (sessionId: string, thinking: boolean) => void;
  toggleSidebar: () => void;
  setActiveTab: (tab: "activity" | "files") => void;
}

export const useAgentStore = create<AgentState>((set) => ({
  sessions: {},
  sidebarVisible: false,
  activeTab: "activity",

  setAgent: (sessionId, identity) =>
    set((state) => ({
      sessions: {
        ...state.sessions,
        [sessionId]: {
          identity,
          events: [],
          fileChanges: [],
          totalInputTokens: 0,
          totalOutputTokens: 0,
          totalCost: 0,
          currentModel: "",
          currentFile: "",
          isThinking: false,
          isEnded: false,
        },
      },
      sidebarVisible: true,
    })),

  endAgent: (sessionId) =>
    set((state) => {
      const session = state.sessions[sessionId];
      if (!session) return state;
      return {
        sessions: {
          ...state.sessions,
          [sessionId]: { ...session, isEnded: true, isThinking: false },
        },
      };
    }),

  addEvent: (sessionId, event) =>
    set((state) => {
      const session = state.sessions[sessionId];
      if (!session) return state;
      return {
        sessions: {
          ...state.sessions,
          [sessionId]: {
            ...session,
            events: [...session.events, event],
          },
        },
      };
    }),

  addFileChange: (sessionId, change) =>
    set((state) => {
      const session = state.sessions[sessionId];
      if (!session) return state;
      return {
        sessions: {
          ...state.sessions,
          [sessionId]: {
            ...session,
            fileChanges: [...session.fileChanges, change],
          },
        },
      };
    }),

  updateTokens: (sessionId, input, output) =>
    set((state) => {
      const session = state.sessions[sessionId];
      if (!session) return state;
      return {
        sessions: {
          ...state.sessions,
          [sessionId]: {
            ...session,
            totalInputTokens: input,
            totalOutputTokens: output,
          },
        },
      };
    }),

  updateCost: (sessionId, cost) =>
    set((state) => {
      const session = state.sessions[sessionId];
      if (!session) return state;
      return {
        sessions: {
          ...state.sessions,
          [sessionId]: { ...session, totalCost: cost },
        },
      };
    }),

  updateModel: (sessionId, model) =>
    set((state) => {
      const session = state.sessions[sessionId];
      if (!session) return state;
      return {
        sessions: {
          ...state.sessions,
          [sessionId]: { ...session, currentModel: model },
        },
      };
    }),

  updateFile: (sessionId, file) =>
    set((state) => {
      const session = state.sessions[sessionId];
      if (!session) return state;
      return {
        sessions: {
          ...state.sessions,
          [sessionId]: { ...session, currentFile: file },
        },
      };
    }),

  setThinking: (sessionId, thinking) =>
    set((state) => {
      const session = state.sessions[sessionId];
      if (!session) return state;
      return {
        sessions: {
          ...state.sessions,
          [sessionId]: { ...session, isThinking: thinking },
        },
      };
    }),

  toggleSidebar: () =>
    set((state) => ({ sidebarVisible: !state.sidebarVisible })),

  setActiveTab: (tab) => set({ activeTab: tab }),
}));
