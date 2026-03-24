import React, { useRef, useEffect, useMemo } from "react";
import { Copy } from "lucide-react";
import type { CellData, GridSnapshot, BlockInfo, CapturedBlock } from "../../stores/terminalStore";
import { colorToCss } from "../../utils/colors";
import { parseAnsi, spanStyle } from "../../utils/ansiParser";
import { detectUrls } from "../../utils/urlDetector";

interface BlocksViewProps {
  snapshot: GridSnapshot;
  onRerun?: (command: string) => void;
  selectedBlockIndex: number | null;
  onSelectBlock?: (index: number) => void;
  capturedBlocks?: CapturedBlock[];
  agentMode?: boolean;
  filterChrome?: boolean;
}

/** Strip ANSI escape codes to get plain text for pattern matching */
function stripAnsi(text: string): string {
  // eslint-disable-next-line no-control-regex
  return text.replace(/\x1b\[[0-9;]*[a-zA-Z]/g, "").replace(/\x1b\][^\x07]*\x07/g, "");
}

/** Lines matching these patterns are Claude Code TUI chrome and should be hidden */
function isAgentChromeLine(line: string): boolean {
  // Strip ANSI codes first so patterns match plain text
  const plain = stripAnsi(line).trim();
  if (!plain) return false;
  // Box-drawing separator lines: ─────── or ────── some-text ──
  if (/^[─━═╌╍┄┅╭╮╰╯│]{10,}/.test(plain)) return true;
  // Lines that are MOSTLY box-drawing (>50% of non-space chars)
  const nonSpace = plain.replace(/\s/g, "");
  const boxChars = (nonSpace.match(/[─━═╌╍┄┅│╭╮╰╯]/g) || []).length;
  if (boxChars > 0 && nonSpace.length > 5 && boxChars / nonSpace.length > 0.5) return true;
  // Prompt indicator line: ❯ with optional trailing text
  if (/^❯/.test(plain) && plain.length < 80) return true;
  // Help hint: ? for shortcuts
  if (/^\?\s+for\s+shortcuts/i.test(plain)) return true;
  // File context indicator: ⧉ In filename
  if (/⧉/.test(plain)) return true;
  // Combined line: "? for shortcuts ... ⧉ In file"
  if (/for\s+shortcuts/i.test(plain) && /In\s+\S+/.test(plain)) return true;
  // Accept edits hint line
  if (/⏵/.test(plain) && /accept/i.test(plain)) return true;
  if (/shift\+tab/i.test(plain) && /cycle/i.test(plain)) return true;
  return false;
}

/** Strip agent TUI chrome lines from output text */
function stripAgentChrome(text: string): string {
  return text
    .split("\n")
    .filter((line) => !isAgentChromeLine(line))
    .join("\n");
}

/** Check if a command is an agent CLI (for early filtering before agent detection) */
const AGENT_COMMANDS = ["claude", "aider", "codex", "ghcs", "github-copilot-cli"];
function isAgentCommand(command: string): boolean {
  const cmd = command.trim().split(/\s+/)[0]?.toLowerCase() ?? "";
  return AGENT_COMMANDS.some((a) => cmd === a || cmd.endsWith("/" + a) || cmd.endsWith("\\" + a));
}

export function BlocksView({
  snapshot,
  onRerun,
  selectedBlockIndex,
  onSelectBlock,
  capturedBlocks,
  agentMode,
  filterChrome = true,
}: BlocksViewProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const prevBlockCountRef = useRef(0);

  // Auto-scroll when new blocks or output appear
  const blockCount = capturedBlocks?.length ?? snapshot.blocks.length;
  const lastOutput = capturedBlocks?.[capturedBlocks.length - 1]?.outputText;
  useEffect(() => {
    if (blockCount > prevBlockCountRef.current || lastOutput) {
      scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
    }
    prevBlockCountRef.current = blockCount;
  }, [blockCount, lastOutput, snapshot.cursor_row]);

  const hasRustBlocks = snapshot.blocks.length > 0;
  const hasCapturedBlocks = (capturedBlocks?.length ?? 0) > 0;

  return (
    <div
      ref={scrollRef}
      className="flex-1 overflow-y-auto"
      style={{
        background: "var(--term-bg)",
        color: "var(--term-fg)",
        fontFamily: "var(--font-mono)",
        fontSize: 14,
        lineHeight: "18px",
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
                agentMode={agentMode}
                filterChrome={filterChrome}
              />
            </React.Fragment>
          );
        })
      ) : hasCapturedBlocks ? (
        <>
          {capturedBlocks!.map((block, idx) => {
            const isSelected = selectedBlockIndex === idx;
            return (
              <React.Fragment key={block.id}>
                {idx > 0 && <Divider />}
                <CapturedOutputBlock
                  block={block}
                  isSelected={isSelected}
                  onSelect={() => onSelectBlock?.(idx)}
                  onRerun={onRerun}
                  agentMode={agentMode}
                  filterChrome={filterChrome}
                />
              </React.Fragment>
            );
          })}
        </>
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

/** Open a URL using window.open as a safe cross-platform fallback. */
function openUrl(url: string): void {
  window.open(url, "_blank");
}

/** Render text with ANSI SGR color codes as styled spans, with clickable URLs. */
const AnsiOutput = React.memo(function AnsiOutput({ text }: { text: string }) {
  const spans = useMemo(() => parseAnsi(text), [text]);
  return (
    <pre
      style={{
        color: "var(--term-fg)",
        margin: 0,
        whiteSpace: "pre",
        overflowX: "auto",
        fontFamily: "inherit",
        fontSize: "inherit",
        lineHeight: "inherit",
      }}
    >
      {spans.map((span, i) => {
        const style = spanStyle(span);
        const urlSegments = detectUrls(span.text);
        const hasUrls = urlSegments.some((s) => s.isUrl);

        if (!hasUrls) {
          return Object.keys(style).length > 0 ? (
            <span key={i} style={style}>
              {span.text}
            </span>
          ) : (
            <React.Fragment key={i}>{span.text}</React.Fragment>
          );
        }

        return (
          <React.Fragment key={i}>
            {urlSegments.map((seg, j) =>
              seg.isUrl ? (
                <span
                  key={j}
                  className="underline cursor-pointer text-blue-400 hover:text-blue-300"
                  style={{ ...style, textDecoration: "underline", cursor: "pointer" }}
                  onClick={(e: React.MouseEvent) => {
                    if ((e.ctrlKey || e.metaKey) && seg.url) {
                      e.preventDefault();
                      e.stopPropagation();
                      openUrl(seg.url);
                    }
                  }}
                  title={`Ctrl+Click to open: ${seg.url}`}
                >
                  {seg.text}
                </span>
              ) : Object.keys(style).length > 0 ? (
                <span key={j} style={style}>
                  {seg.text}
                </span>
              ) : (
                <React.Fragment key={j}>{seg.text}</React.Fragment>
              ),
            )}
          </React.Fragment>
        );
      })}
    </pre>
  );
});

/** Block with captured plain-text output from Rust-side PTY capture. */
function CapturedOutputBlock({
  block,
  isSelected,
  onSelect,
  onRerun,
  agentMode,
  filterChrome = true,
}: {
  block: CapturedBlock;
  isSelected: boolean;
  onSelect?: () => void;
  onRerun?: (command: string) => void;
  agentMode?: boolean;
  filterChrome?: boolean;
}) {
  const blockRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (isSelected && blockRef.current) {
      blockRef.current.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  }, [isSelected]);

  const handleCopy = () => {
    navigator.clipboard.writeText(block.outputText);
  };

  // Format duration
  const durationStr = block.durationMs != null
    ? block.durationMs >= 1000
      ? `${(block.durationMs / 1000).toFixed(3)}s`
      : `${block.durationMs}ms`
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
          {block.cwd || ""}
          {block.gitBranch && (
            <span style={{ color: "var(--color-accent)" }}>{" "}git:{block.gitBranch}</span>
          )}
        </span>
        {durationStr && (
          <span style={{ color: "var(--color-text-muted)", fontSize: 11 }}>
            ({durationStr})
          </span>
        )}
        {block.outputText && (
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
        )}
      </div>

      {/* Command line */}
      <div style={{ marginBottom: 4 }}>
        <span
          style={{ color: "var(--color-text)", fontWeight: 500, cursor: "pointer" }}
          onClick={() => onRerun?.(block.command)}
          title="Click to re-run"
        >
          {block.command}
        </span>
      </div>

      {/* Output (colored, with agent TUI chrome stripped when enabled) */}
      {block.outputText ? (
        <AnsiOutput text={(filterChrome && (agentMode || isAgentCommand(block.command))) ? stripAgentChrome(block.outputText) : block.outputText} />
      ) : !block.durationMs ? (
        <div style={{ color: "var(--color-text-muted)", fontSize: 12, opacity: 0.6 }}>
          ...
        </div>
      ) : null}
    </div>
  );
}

/** Rust-side block (shell integration via OSC 133) */
function RustBlock({
  block,
  rows,
  isSelected,
  onSelect,
  onRerun,
  agentMode,
  filterChrome = true,
}: {
  block: BlockInfo;
  rows: CellData[][];
  isSelected: boolean;
  onSelect?: () => void;
  onRerun?: (command: string) => void;
  agentMode?: boolean;
  filterChrome?: boolean;
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
    let filtered = outputRows;
    // In agent mode (or for agent commands), filter out TUI chrome lines if enabled
    if (filterChrome && (agentMode || isAgentCommand(block.command))) {
      filtered = filtered.filter((row) => {
        const text = row.map((c) => c.content || "").join("");
        return !isAgentChromeLine(text);
      });
    }
    let end = filtered.length;
    while (end > 0) {
      const text = filtered[end - 1].map((c) => c.content || "").join("").trim();
      if (text) break;
      end--;
    }
    return filtered.slice(0, end);
  }, [outputRows, agentMode]);

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

      {trimmedRows.length > 0 && (
        <div
          className="whitespace-pre"
          style={{
            color: isError ? "var(--color-error)" : "var(--term-fg)",
            lineHeight: "18px",
            overflowX: "auto",
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
  if (!line) return <div style={{ height: 18 }} />;

  return (
    <div style={{ height: 18, lineHeight: "18px" }}>
      {spans.map((span, i) => {
        const urlSegments = detectUrls(span.text);
        const hasUrls = urlSegments.some((s) => s.isUrl);

        if (!hasUrls) {
          return (
            <span key={i} style={span.style}>
              {span.text}
            </span>
          );
        }

        return (
          <React.Fragment key={i}>
            {urlSegments.map((seg, j) =>
              seg.isUrl ? (
                <span
                  key={j}
                  className="underline cursor-pointer text-blue-400 hover:text-blue-300"
                  style={{ ...span.style, textDecoration: "underline", cursor: "pointer" }}
                  onClick={(e: React.MouseEvent) => {
                    if ((e.ctrlKey || e.metaKey) && seg.url) {
                      e.preventDefault();
                      e.stopPropagation();
                      openUrl(seg.url);
                    }
                  }}
                  title={`Ctrl+Click to open: ${seg.url}`}
                >
                  {seg.text}
                </span>
              ) : (
                <span key={j} style={span.style}>
                  {seg.text}
                </span>
              ),
            )}
          </React.Fragment>
        );
      })}
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
