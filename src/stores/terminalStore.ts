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

/** Captured command block — output is plain text from Rust-side PTY capture. */
export interface CapturedBlock {
  /** Unique command ID (matches Rust-side command_id). */
  id: number;
  command: string;
  cwd: string;
  gitBranch?: string;
  submittedAt: number;
  /** Plain text output (set when command_output event arrives). */
  outputText: string;
  /** Timestamp of the last output chunk received. */
  lastChunkAt?: number;
  /** Duration in ms (set when finalized). */
  durationMs?: number;
}

let nextCommandId = 1;

/** Generate a unique command ID for submit_command. */
export function getNextCommandId(): number {
  return nextCommandId++;
}

interface TerminalState {
  grids: Map<string, GridSnapshot>;
  selectedBlockIndex: number | null;
  /** Captured command blocks per session. */
  capturedBlocks: Map<string, CapturedBlock[]>;

  updateGrid: (sessionId: string, snapshot: GridSnapshot) => void;
  selectBlock: (index: number | null) => void;
  moveBlockSelection: (direction: "up" | "down", blockCount: number) => void;

  /** Add a new captured block when user submits a command. */
  addCapturedBlock: (
    sessionId: string,
    commandId: number,
    command: string,
    cwd: string,
    gitBranch?: string,
  ) => void;
  /** Update a captured block with live streaming output. */
  updateOutputChunk: (
    sessionId: string,
    commandId: number,
    output: string,
  ) => void;
  /** Finalize a captured block with output from Rust. */
  finalizeOutput: (
    sessionId: string,
    commandId: number,
    output: string,
    durationMs: number,
  ) => void;
  /** Clear captured blocks for a session. */
  clearCapturedBlocks: (sessionId: string) => void;
}

export const useTerminalStore = create<TerminalState>((set) => ({
  grids: new Map(),
  selectedBlockIndex: null,
  capturedBlocks: new Map(),

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

  addCapturedBlock: (sessionId, commandId, command, cwd, gitBranch) =>
    set((state) => {
      const capturedBlocks = new Map(state.capturedBlocks);
      const blocks = [...(capturedBlocks.get(sessionId) || [])];
      blocks.push({
        id: commandId,
        command,
        cwd,
        gitBranch,
        submittedAt: Date.now(),
        outputText: "",
      });
      capturedBlocks.set(sessionId, blocks);
      return { capturedBlocks };
    }),

  updateOutputChunk: (sessionId, commandId, output) =>
    set((state) => {
      const capturedBlocks = new Map(state.capturedBlocks);
      const blocks = capturedBlocks.get(sessionId);
      if (!blocks) return state;
      const now = Date.now();
      // Only update if not yet finalized
      const updated = blocks.map((b) =>
        b.id === commandId && b.durationMs == null
          ? { ...b, outputText: output, lastChunkAt: now }
          : b,
      );
      capturedBlocks.set(sessionId, updated);
      return { capturedBlocks };
    }),

  finalizeOutput: (sessionId, commandId, output, durationMs) =>
    set((state) => {
      const capturedBlocks = new Map(state.capturedBlocks);
      const blocks = capturedBlocks.get(sessionId);
      if (!blocks) return state;
      const updated = blocks.map((b) => {
        if (b.id !== commandId) return b;
        // Don't overwrite if already finalized (auto-finalize was more accurate)
        if (b.durationMs != null) return { ...b, outputText: output };
        return { ...b, outputText: output, durationMs };
      });
      capturedBlocks.set(sessionId, updated);
      return { capturedBlocks };
    }),

  clearCapturedBlocks: (sessionId) =>
    set((state) => {
      const capturedBlocks = new Map(state.capturedBlocks);
      capturedBlocks.delete(sessionId);
      return { capturedBlocks };
    }),
}));
