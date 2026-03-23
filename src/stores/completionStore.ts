import { create } from "zustand";

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
  show: (items: CompletionItem[]) => void;
  hide: () => void;
  selectNext: () => void;
  selectPrevious: () => void;
  getSelected: () => CompletionItem | undefined;
}

export const useCompletionStore = create<CompletionState>((set, get) => ({
  visible: false,
  items: [],
  selectedIndex: 0,
  show: (items) => set({ visible: true, items, selectedIndex: 0 }),
  hide: () => set({ visible: false, items: [], selectedIndex: 0 }),
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
}));
