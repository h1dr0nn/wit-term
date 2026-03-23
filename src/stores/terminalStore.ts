import { create } from "zustand";

export interface CellData {
  content: string;
  fg: ColorData;
  bg: ColorData;
  bold: boolean;
  dim: boolean;
  italic: boolean;
  underline: boolean;
  strikethrough: boolean;
  inverse: boolean;
}

export type ColorData =
  | { type: "Default" }
  | { type: "Named"; name: string }
  | { type: "Indexed"; index: number }
  | { type: "Rgb"; r: number; g: number; b: number };

export interface BlockInfo {
  id: number;
  prompt_row: number;
  output_start_row: number | null;
  output_end_row: number | null;
  exit_code: number | null;
  cwd: string;
  command: string;
}

export interface GridSnapshot {
  rows: CellData[][];
  cursor_row: number;
  cursor_col: number;
  cursor_visible: boolean;
  cursor_shape: "Block" | "Underline" | "Bar";
  blocks: BlockInfo[];
}

interface TerminalState {
  grids: Map<string, GridSnapshot>;
  updateGrid: (sessionId: string, snapshot: GridSnapshot) => void;
}

export const useTerminalStore = create<TerminalState>((set) => ({
  grids: new Map(),
  updateGrid: (sessionId, snapshot) =>
    set((state) => {
      const grids = new Map(state.grids);
      grids.set(sessionId, snapshot);
      return { grids };
    }),
}));
