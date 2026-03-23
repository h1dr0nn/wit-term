import React, { useRef, useEffect, useMemo, useState, useCallback } from "react";
import {
  CheckCircle,
  XCircle,
  Loader,
  ChevronDown,
  ChevronRight,
  Folder,
  Copy,
  RotateCw,
} from "lucide-react";
import type { CellData, GridSnapshot, BlockInfo } from "../../stores/terminalStore";
import { colorToCss } from "../../utils/colors";

interface BlocksViewProps {
  snapshot: GridSnapshot;
  onRerun?: (command: string) => void;
}

export function BlocksView({ snapshot, onRerun }: BlocksViewProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const prevBlockCountRef = useRef(0);

  // Auto-scroll to bottom when new blocks appear
  useEffect(() => {
    if (snapshot.blocks.length > prevBlockCountRef.current) {
      scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
    }
    prevBlockCountRef.current = snapshot.blocks.length;
  }, [snapshot.blocks.length]);

  // If no blocks yet (no shell integration), fall back to raw grid
  if (snapshot.blocks.length === 0) {
    return <RawGrid snapshot={snapshot} scrollRef={scrollRef} />;
  }

  return (
    <div
      ref={scrollRef}
      className="flex-1 overflow-y-auto"
      style={{ padding: "var(--sp-2) var(--sp-3)" }}
    >
      <div className="flex flex-col gap-2">
        {snapshot.blocks.map((block, idx) => {
          const isLast = idx === snapshot.blocks.length - 1;
          const isRunning = isLast && block.exit_code === null && block.output_start_row !== null;
          return (
            <CommandBlock
              key={block.id}
              block={block}
              rows={snapshot.rows}
              isRunning={isRunning}
              onRerun={onRerun}
            />
          );
        })}
      </div>
    </div>
  );
}

/** Fallback: raw grid rendering when no blocks detected */
function RawGrid({
  snapshot,
  scrollRef,
}: {
  snapshot: GridSnapshot;
  scrollRef: React.RefObject<HTMLDivElement | null>;
}) {
  // Auto-scroll to cursor
  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
  }, [snapshot.cursor_row, scrollRef]);

  return (
    <div
      ref={scrollRef}
      className="flex-1 overflow-y-auto font-mono text-sm leading-[1.3] whitespace-pre"
      style={{ padding: "var(--sp-2)" }}
    >
      {snapshot.rows.map((row, i) => (
        <TerminalRow key={i} cells={row} />
      ))}
    </div>
  );
}

/** A single command block */
interface CommandBlockProps {
  block: BlockInfo;
  rows: CellData[][];
  isRunning: boolean;
  onRerun?: (command: string) => void;
}

function CommandBlock({ block, rows, isRunning, onRerun }: CommandBlockProps) {
  const [collapsed, setCollapsed] = useState(false);

  const outputRows = useMemo(() => {
    if (block.output_start_row === null) return [];
    const start = block.output_start_row;
    const end = block.output_end_row ?? rows.length;
    return rows.slice(start, end);
  }, [block, rows]);

  const hasOutput = outputRows.length > 0;
  const isSuccess = block.exit_code === 0;
  const isError = block.exit_code !== null && block.exit_code !== 0;
  const cwdDisplay = cwdShort(block.cwd);

  const handleCopy = useCallback(() => {
    const text = outputRows
      .map((row) => row.map((c) => c.content || " ").join("").trimEnd())
      .join("\n");
    navigator.clipboard.writeText(text);
  }, [outputRows]);

  const handleRerun = useCallback(() => {
    if (onRerun && block.command) {
      onRerun(block.command);
    }
  }, [onRerun, block.command]);

  return (
    <div
      style={{
        borderRadius: "var(--radius-lg)",
        border: `1px solid ${isError ? "var(--color-error)" : isRunning ? "var(--color-primary)" : "var(--color-border-muted)"}`,
        background: "var(--color-surface)",
        overflow: "hidden",
        opacity: isError ? 1 : 1,
      }}
    >
      {/* Block header */}
      <div
        className="flex items-center gap-2 group cursor-pointer"
        style={{
          padding: "6px 10px",
          background: isError
            ? "rgba(248, 81, 73, 0.06)"
            : isRunning
              ? "rgba(88, 230, 217, 0.04)"
              : "transparent",
        }}
        onClick={() => hasOutput && setCollapsed(!collapsed)}
      >
        {/* Expand/collapse toggle */}
        {hasOutput && (
          <span style={{ color: "var(--color-text-muted)" }}>
            {collapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
          </span>
        )}

        {/* Status icon */}
        {isRunning && <Loader size={14} className="animate-spin" style={{ color: "var(--color-primary)" }} />}
        {isSuccess && <CheckCircle size={14} style={{ color: "var(--color-success)" }} />}
        {isError && <XCircle size={14} style={{ color: "var(--color-error)" }} />}

        {/* CWD */}
        <span
          style={{
            fontSize: 11,
            color: "var(--color-text-muted)",
            fontFamily: "var(--font-mono)",
          }}
          className="flex items-center gap-1"
        >
          <Folder size={10} />
          {cwdDisplay}
        </span>

        {/* Command text */}
        <span
          style={{
            fontSize: 13,
            color: "var(--color-text)",
            fontFamily: "var(--font-mono)",
            fontWeight: 500,
          }}
          className="flex-1 truncate"
        >
          {block.command || "..."}
        </span>

        {/* Action buttons (visible on hover) */}
        <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
          {hasOutput && (
            <button
              onClick={(e) => { e.stopPropagation(); handleCopy(); }}
              title="Copy output"
              style={{ width: 24, height: 24, borderRadius: "var(--radius-sm)", color: "var(--color-text-muted)" }}
              className="flex items-center justify-center hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
            >
              <Copy size={12} />
            </button>
          )}
          {block.command && (
            <button
              onClick={(e) => { e.stopPropagation(); handleRerun(); }}
              title="Re-run command"
              style={{ width: 24, height: 24, borderRadius: "var(--radius-sm)", color: "var(--color-text-muted)" }}
              className="flex items-center justify-center hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
            >
              <RotateCw size={12} />
            </button>
          )}
        </div>

        {/* Exit code badge */}
        {isError && (
          <span
            style={{
              fontSize: 10,
              color: "var(--color-error)",
              background: "rgba(248, 81, 73, 0.1)",
              padding: "1px 6px",
              borderRadius: "var(--radius-xs)",
              fontFamily: "var(--font-mono)",
            }}
          >
            {block.exit_code}
          </span>
        )}
      </div>

      {/* Block output */}
      {hasOutput && !collapsed && (
        <div
          style={{
            borderTop: "1px solid var(--color-border-muted)",
            padding: "var(--sp-2) var(--sp-3)",
            maxHeight: 400,
            overflow: "auto",
            fontFamily: "var(--font-mono)",
            fontSize: 13,
            lineHeight: "1.3",
          }}
          className="whitespace-pre"
        >
          {outputRows.map((row, i) => (
            <TerminalRow key={i} cells={row} />
          ))}
        </div>
      )}
    </div>
  );
}

/** Render a single row of terminal cells */
const TerminalRow = React.memo(function TerminalRow({ cells }: { cells: CellData[] }) {
  // Group consecutive cells with same style into spans
  const spans = useMemo(() => {
    const result: { text: string; style: React.CSSProperties }[] = [];
    let currentCells: CellData[] = [];

    for (const cell of cells) {
      if (currentCells.length > 0 && !sameStyle(currentCells[0], cell)) {
        result.push({
          text: currentCells.map((c) => c.content || " ").join(""),
          style: cellStyle(currentCells[0]),
        });
        currentCells = [];
      }
      currentCells.push(cell);
    }
    if (currentCells.length > 0) {
      result.push({
        text: currentCells.map((c) => c.content || " ").join(""),
        style: cellStyle(currentCells[0]),
      });
    }
    return result;
  }, [cells]);

  const line = spans
    .map((s) => s.text)
    .join("")
    .trimEnd();
  if (!line) return <div style={{ height: "1.3em" }} />;

  return (
    <div>
      {spans.map((span, i) => (
        <span key={i} style={span.style}>
          {span.text}
        </span>
      ))}
    </div>
  );
});

function sameStyle(a: CellData, b: CellData): boolean {
  return (
    a.bold === b.bold &&
    a.dim === b.dim &&
    a.italic === b.italic &&
    a.underline === b.underline &&
    a.strikethrough === b.strikethrough &&
    a.inverse === b.inverse &&
    JSON.stringify(a.fg) === JSON.stringify(b.fg) &&
    JSON.stringify(a.bg) === JSON.stringify(b.bg)
  );
}

function cellStyle(cell: CellData): React.CSSProperties {
  const style: React.CSSProperties = {};
  const fg = cell.inverse ? cell.bg : cell.fg;
  const bg = cell.inverse ? cell.fg : cell.bg;
  const fgColor = colorToCss(fg, true);
  const bgColor = colorToCss(bg, false);
  if (fgColor) style.color = fgColor;
  if (bgColor) style.backgroundColor = bgColor;
  if (cell.bold) style.fontWeight = "bold";
  if (cell.dim) style.opacity = 0.5;
  if (cell.italic) style.fontStyle = "italic";
  const deco: string[] = [];
  if (cell.underline) deco.push("underline");
  if (cell.strikethrough) deco.push("line-through");
  if (deco.length) style.textDecoration = deco.join(" ");
  return style;
}

function cwdShort(cwd: string): string {
  if (!cwd) return "~";
  const parts = cwd.replace(/\\/g, "/").split("/").filter(Boolean);
  if (parts.length <= 2) return cwd.replace(/\\/g, "/");
  return "~/" + parts.slice(-2).join("/");
}
