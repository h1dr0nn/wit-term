import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

interface ContextInfo {
  provider: string;
  data: Record<string, unknown>;
  detected_markers?: string[];
}

interface ProjectContext {
  project_root: string | null;
  cwd: string;
  providers: Record<string, ContextInfo>;
  last_updated: number;
  completion_sets: string[];
}

interface ContextState {
  context: ProjectContext | null;
  loading: boolean;

  fetchContext: (cwd: string) => Promise<void>;
  clearContext: () => void;
  initListener: () => Promise<UnlistenFn>;
}

export const useContextStore = create<ContextState>((set) => ({
  context: null,
  loading: false,

  fetchContext: async (cwd: string) => {
    set({ loading: true });
    try {
      const context = await invoke<ProjectContext>("get_context", { cwd });
      set({ context, loading: false });
    } catch {
      set({ context: null, loading: false });
    }
  },

  clearContext: () => set({ context: null }),

  initListener: async () => {
    const unlisten = await listen<{ session_id: string; context: ProjectContext }>(
      "context_changed",
      (event) => {
        set({ context: event.payload.context });
      },
    );
    return unlisten;
  },
}));
