import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface CompletionItem {
  text: string;
  display: string;
  description: string;
  kind: "Command" | "Subcommand" | "Flag" | "Argument" | "Path";
  score: number;
}

interface CompletionState {
  visible: boolean;
  items: CompletionItem[];
  selectedIndex: number;
  position: { x: number; y: number };
  inlineHint: string | null;

  show: (items: CompletionItem[], position?: { x: number; y: number }) => void;
  hide: () => void;
  selectNext: () => void;
  selectPrevious: () => void;
  getSelected: () => CompletionItem | undefined;
  setInlineHint: (hint: string | null) => void;
  accept: (sessionId: string) => Promise<void>;
}

export const useCompletionStore = create<CompletionState>((set, get) => ({
  visible: false,
  items: [],
  selectedIndex: 0,
  position: { x: 0, y: 0 },
  inlineHint: null,

  show: (items, position) =>
    set({
      visible: true,
      items,
      selectedIndex: 0,
      ...(position ? { position } : {}),
    }),
  hide: () => set({ visible: false, items: [], selectedIndex: 0, inlineHint: null }),
  selectNext: () =>
    set((state) => ({
      selectedIndex: (state.selectedIndex + 1) % Math.max(1, state.items.length),
    })),
  selectPrevious: () =>
    set((state) => ({
      selectedIndex:
        (state.selectedIndex - 1 + state.items.length) % Math.max(1, state.items.length),
    })),
  getSelected: () => {
    const state = get();
    return state.items[state.selectedIndex];
  },
  setInlineHint: (hint) => set({ inlineHint: hint }),
  accept: async (sessionId: string) => {
    const selected = get().getSelected();
    if (selected) {
      try {
        await invoke("accept_completion", {
          sessionId,
          text: selected.text,
        });
      } catch {
        // Silent fail
      }
      set({ visible: false, items: [], selectedIndex: 0, inlineHint: null });
    }
  },
}));
