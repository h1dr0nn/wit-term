import React, { useRef, useEffect, useMemo } from "react";
import { Copy } from "lucide-react";
import type { CellData, GridSnapshot, BlockInfo, FrontendBlock } from "../../stores/terminalStore";
import { colorToCss } from "../../utils/colors";

interface BlocksViewProps {
  snapshot: GridSnapshot;
  onRerun?: (command: string) => void;
  selectedBlockIndex: number | null;
  onSelectBlock?: (index: number) => void;
  frontendBlocks?: FrontendBlock[];
}

export function BlocksView({
  snapshot,
  onRerun,
  selectedBlockIndex,
  onSelectBlock,
  frontendBlocks,
}: BlocksViewProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const prevRowCountRef = useRef(0);

  // Auto-scroll when new output appears
  useEffect(() => {
    const rowCount = snapshot.rows.length;
    if (rowCount > prevRowCountRef.current) {
      scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
    }
    prevRowCountRef.current = rowCount;
  }, [snapshot.rows.length]);

  const hasRustBlocks = snapshot.blocks.length > 0;
  const hasFrontendBlocks = (frontendBlocks?.length ?? 0) > 0;

  return (
    <div
      ref={scrollRef}
      className="flex-1 overflow-y-auto"
      style={{
        background: "var(--term-bg)",
        color: "var(--term-fg)",
        fontFamily: "var(--font-mono)",
        fontSize: 14,
        lineHeight: "20px",
        padding: "var(--sp-3) var(--sp-4)",
      }}
    >
      {hasRustBlocks ? (
        snapshot.blocks.map((block, idx) => {
          const isSelected = selectedBlockIndex === idx;
          return (
            <React.Fragment key={block.id}>
              {idx > 0 && <Divider />}
              <RustBlock
                block={block}
                rows={snapshot.rows}
                isSelected={isSelected}
                onSelect={() => onSelectBlock?.(idx)}
                onRerun={onRerun}
              />
            </React.Fragment>
          );
        })
      ) : hasFrontendBlocks ? (
        frontendBlocks!.map((fb, idx) => {
          const isSelected = selectedBlockIndex === idx;
          const startRow = fb.gridRowStart;
          const endRow = fb.gridRowEnd ?? snapshot.rows.length;
          const outputRows = snapshot.rows.slice(startRow, endRow);

          // Calculate duration if next block exists or command is finished
          const nextBlock = frontendBlocks![idx + 1];
          const durationMs = nextBlock
            ? nextBlock.submittedAt - fb.submittedAt
            : undefined;

          return (
            <React.Fragment key={fb.id}>
              {idx > 0 && <Divider />}
              <FlatBlock
                command={fb.command}
                outputRows={outputRows}
                isSelected={isSelected}
                onSelect={() => onSelectBlock?.(idx)}
                onRerun={onRerun}
                cwd={fb.cwd}
                gitBranch={fb.gitBranch}
                durationMs={durationMs}
              />
            </React.Fragment>
          );
        })
      ) : (
        <div
          style={{
            color: "var(--color-text-muted)",
            fontSize: 13,
            paddingTop: 60,
            textAlign: "center",
          }}
        >
        </div>
      )}
    </div>
  );
}

/** Horizontal divider between blocks */
function Divider() {
  return (
    <div
      style={{
        height: 1,
        background: "var(--color-border-muted)",
        margin: "12px 0",
      }}
    />
  );
}

/** Flat block for frontend-driven blocks (no shell integration) */
function FlatBlock({
  command,
  outputRows,
  isSelected,
  onSelect,
  onRerun,
  cwd,
  gitBranch,
  durationMs,
}: {
  command: string;
  outputRows: CellData[][];
  isSelected: boolean;
  onSelect?: () => void;
  onRerun?: (command: string) => void;
  cwd?: string;
  gitBranch?: string;
  durationMs?: number;
}) {
  const blockRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (isSelected && blockRef.current) {
      blockRef.current.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  }, [isSelected]);

  // Trim trailing empty rows
  const trimmedRows = useMemo(() => {
    let end = outputRows.length;
    while (end > 0) {
      const text = outputRows[end - 1].map((c) => c.content || "").join("").trim();
      if (text) break;
      end--;
    }
    return outputRows.slice(0, end);
  }, [outputRows]);

  const handleCopy = () => {
    const text = trimmedRows
      .map((row) => row.map((c) => c.content || " ").join("").trimEnd())
      .join("\n");
    navigator.clipboard.writeText(text);
  };

  // Format duration
  const durationStr = durationMs != null
    ? durationMs >= 1000
      ? `${(durationMs / 1000).toFixed(3)}s`
      : `${durationMs}ms`
    : null;

  return (
    <div
      ref={blockRef}
      onClick={onSelect}
      style={{
        borderLeft: isSelected ? "2px solid var(--color-primary)" : "2px solid transparent",
        paddingLeft: 10,
      }}
    >
      {/* Header: CWD + git branch + duration (dimmed) */}
      <div
        className="flex items-center gap-2 group"
        style={{
          opacity: 0.5,
          fontSize: 12,
          lineHeight: "18px",
          marginBottom: 2,
        }}
      >
        <span style={{ color: "var(--color-text-muted)", flex: 1 }}>
          {cwd || ""}
          {gitBranch && (
            <span style={{ color: "var(--color-accent)" }}>{" "}git:{gitBranch}</span>
          )}
        </span>
        {durationStr && (
          <span style={{ color: "var(--color-text-muted)", fontSize: 11 }}>
            ({durationStr})
          </span>
        )}
        <button
          onClick={(e) => { e.stopPropagation(); handleCopy(); }}
          title="Copy output"
          className="opacity-0 group-hover:opacity-100 transition-opacity"
          style={{
            background: "none",
            border: "none",
            color: "var(--color-text-muted)",
            cursor: "pointer",
            padding: 2,
          }}
        >
          <Copy size={12} />
        </button>
      </div>

      {/* Command line */}
      <div style={{ marginBottom: 4 }}>
        <span
          style={{ color: "var(--color-text)", fontWeight: 500, cursor: "pointer" }}
          onClick={() => onRerun?.(command)}
          title="Click to re-run"
        >
          {command}
        </span>
      </div>

      {/* Output */}
      {trimmedRows.length > 0 && (
        <div className="whitespace-pre" style={{ color: "var(--term-fg)" }}>
          {trimmedRows.map((row, i) => (
            <TerminalRow key={i} cells={row} />
          ))}
        </div>
      )}
    </div>
  );
}

/** Flat block for Rust-side blocks (shell integration active) */
function RustBlock({
  block,
  rows,
  isSelected,
  onSelect,
  onRerun,
}: {
  block: BlockInfo;
  rows: CellData[][];
  isSelected: boolean;
  onSelect?: () => void;
  onRerun?: (command: string) => void;
}) {
  const blockRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (isSelected && blockRef.current) {
      blockRef.current.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  }, [isSelected]);

  const outputRows = useMemo(() => {
    if (block.output_start_row === null) return [];
    const start = block.output_start_row;
    const end = block.output_end_row ?? rows.length;
    return rows.slice(start, end);
  }, [block, rows]);

  const trimmedRows = useMemo(() => {
    let end = outputRows.length;
    while (end > 0) {
      const text = outputRows[end - 1].map((c) => c.content || "").join("").trim();
      if (text) break;
      end--;
    }
    return outputRows.slice(0, end);
  }, [outputRows]);

  const isError = block.exit_code !== null && block.exit_code !== 0;

  const handleCopy = () => {
    const text = trimmedRows
      .map((row) => row.map((c) => c.content || " ").join("").trimEnd())
      .join("\n");
    navigator.clipboard.writeText(text);
  };

  return (
    <div
      ref={blockRef}
      onClick={onSelect}
      style={{
        borderLeft: isSelected
          ? "2px solid var(--color-primary)"
          : isError
            ? "2px solid var(--color-error)"
            : "2px solid transparent",
        paddingLeft: 10,
      }}
    >
      {/* Command line */}
      <div
        className="flex items-center gap-2 group"
        style={{ marginBottom: 4 }}
      >
        <span style={{ color: "var(--color-primary)", fontWeight: 600 }}>$</span>
        <span
          style={{ color: "var(--color-text)", fontWeight: 500, flex: 1, cursor: "pointer" }}
          onClick={() => onRerun?.(block.command)}
          title="Click to re-run"
        >
          {block.command || "..."}
        </span>
        {isError && (
          <span style={{ color: "var(--color-error)", fontSize: 11 }}>
            exit {block.exit_code}
          </span>
        )}
        <button
          onClick={(e) => { e.stopPropagation(); handleCopy(); }}
          title="Copy output"
          className="opacity-0 group-hover:opacity-100 transition-opacity"
          style={{
            background: "none",
            border: "none",
            color: "var(--color-text-muted)",
            cursor: "pointer",
            padding: 2,
          }}
        >
          <Copy size={12} />
        </button>
      </div>

      {/* Output */}
      {trimmedRows.length > 0 && (
        <div
          className="whitespace-pre"
          style={{
            color: isError ? "var(--color-error)" : "var(--term-fg)",
          }}
        >
          {trimmedRows.map((row, i) => (
            <TerminalRow key={i} cells={row} />
          ))}
        </div>
      )}
    </div>
  );
}

/** Render a single row of terminal cells */
const TerminalRow = React.memo(function TerminalRow({ cells }: { cells: CellData[] }) {
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

  const line = spans.map((s) => s.text).join("").trimEnd();
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
