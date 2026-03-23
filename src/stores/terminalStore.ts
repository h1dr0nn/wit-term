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
  /** Number of scrollback rows at the beginning of `rows`. */
  scrollback_len: number;
}

/** Frontend-created block (when shell integration isn't available) */
export interface FrontendBlock {
  id: number;
  command: string;
  cwd: string;
  /** Git branch at time of submission */
  gitBranch?: string;
  /** Grid row count at time of submission (output starts after this) */
  gridRowStart: number;
  /** Set when next command is submitted */
  gridRowEnd: number | null;
  /** Timestamp */
  submittedAt: number;
}

let nextFrontendBlockId = 1;

interface TerminalState {
  grids: Map<string, GridSnapshot>;
  selectedBlockIndex: number | null;
  /** Frontend-driven blocks per session (used when no shell integration) */
  frontendBlocks: Map<string, FrontendBlock[]>;

  updateGrid: (sessionId: string, snapshot: GridSnapshot) => void;
  selectBlock: (index: number | null) => void;
  moveBlockSelection: (direction: "up" | "down", blockCount: number) => void;

  /** Create a frontend block when user submits command from InputBar */
  addFrontendBlock: (sessionId: string, command: string, cwd: string, gridRowCount: number, gitBranch?: string) => void;
  /** Clear frontend blocks for a session */
  clearFrontendBlocks: (sessionId: string) => void;
}

export const useTerminalStore = create<TerminalState>((set) => ({
  grids: new Map(),
  selectedBlockIndex: null,
  frontendBlocks: new Map(),

  updateGrid: (sessionId, snapshot) =>
    set((state) => {
      const grids = new Map(state.grids);
      grids.set(sessionId, snapshot);
      return { grids };
    }),

  selectBlock: (index) => set({ selectedBlockIndex: index }),

  moveBlockSelection: (direction, blockCount) =>
    set((state) => {
      if (blockCount === 0) return { selectedBlockIndex: null };
      const current = state.selectedBlockIndex;
      if (current === null) {
        return { selectedBlockIndex: direction === "up" ? blockCount - 1 : 0 };
      }
      if (direction === "up") {
        return { selectedBlockIndex: Math.max(0, current - 1) };
      }
      const next = current + 1;
      if (next >= blockCount) {
        return { selectedBlockIndex: null };
      }
      return { selectedBlockIndex: next };
    }),

  addFrontendBlock: (sessionId, command, cwd, gridRowCount, gitBranch) =>
    set((state) => {
      const frontendBlocks = new Map(state.frontendBlocks);
      const blocks = [...(frontendBlocks.get(sessionId) || [])];

      // Close the previous block's output range
      if (blocks.length > 0) {
        const last = blocks[blocks.length - 1];
        blocks[blocks.length - 1] = { ...last, gridRowEnd: gridRowCount };
      }

      blocks.push({
        id: nextFrontendBlockId++,
        command,
        cwd,
        gitBranch,
        gridRowStart: gridRowCount,
        gridRowEnd: null,
        submittedAt: Date.now(),
      });

      frontendBlocks.set(sessionId, blocks);
      return { frontendBlocks };
    }),

  clearFrontendBlocks: (sessionId) =>
    set((state) => {
      const frontendBlocks = new Map(state.frontendBlocks);
      frontendBlocks.delete(sessionId);
      return { frontendBlocks };
    }),
}));
