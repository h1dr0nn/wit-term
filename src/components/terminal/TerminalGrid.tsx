import React, { useMemo, useRef, useCallback, useEffect, useState } from "react";
import type { CellData, GridSnapshot } from "../../stores/terminalStore";
import { colorToCss } from "../../utils/colors";

interface TerminalGridProps {
  snapshot: GridSnapshot;
}

const ROW_HEIGHT = 18; // Slightly more spacious for readability
const FONT_SIZE = 14;

export function TerminalGrid({ snapshot }: TerminalGridProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [viewportHeight, setViewportHeight] = useState(0);

  // Track viewport size
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (entry) {
        setViewportHeight(entry.contentRect.height);
      }
    });

    observer.observe(container);
    setViewportHeight(container.clientHeight);
    return () => observer.disconnect();
  }, []);

  const handleScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    setScrollTop(e.currentTarget.scrollTop);
  }, []);

  // Virtual scrolling: only render visible rows
  const totalHeight = snapshot.rows.length * ROW_HEIGHT;
  const startRow = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - 2);
  const endRow = Math.min(
    snapshot.rows.length,
    Math.ceil((scrollTop + viewportHeight) / ROW_HEIGHT) + 2,
  );

  // Memoize visible rows
  const visibleRows = useMemo(() => {
    const rows = [];
    for (let i = startRow; i < endRow; i++) {
      rows.push(
        <TerminalRow
          key={i}
          cells={snapshot.rows[i]}
          rowIdx={i}
          cursorCol={snapshot.cursor_col}
          showCursor={snapshot.cursor_visible && i === snapshot.cursor_row}
          cursorShape={snapshot.cursor_shape}
          top={i * ROW_HEIGHT}
        />,
      );
    }
    return rows;
  }, [snapshot, startRow, endRow]);

  // Auto-scroll to cursor
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const cursorY = snapshot.cursor_row * ROW_HEIGHT;
    const viewBottom = container.scrollTop + container.clientHeight;

    if (cursorY + ROW_HEIGHT > viewBottom) {
      container.scrollTop = cursorY + ROW_HEIGHT - container.clientHeight;
    } else if (cursorY < container.scrollTop) {
      container.scrollTop = cursorY;
    }
  }, [snapshot.cursor_row]);

  return (
    <div
      ref={containerRef}
      className="flex-1 overflow-auto whitespace-pre"
      onScroll={handleScroll}
      style={{
        background: "var(--term-bg)",
        color: "var(--term-fg)",
        fontFamily: "var(--font-mono)",
        fontSize: FONT_SIZE,
        lineHeight: `${ROW_HEIGHT}px`,
        padding: "var(--sp-2) var(--sp-3)",
      }}
    >
      <div style={{ height: totalHeight, position: "relative" }}>
        {visibleRows}
      </div>
    </div>
  );
}

interface TerminalRowProps {
  cells: CellData[];
  rowIdx: number;
  cursorCol: number;
  showCursor: boolean;
  cursorShape: string;
  top: number;
}

const TerminalRow = React.memo(function TerminalRow({
  cells,
  cursorCol,
  showCursor,
  cursorShape,
  top,
}: TerminalRowProps) {
  // Group consecutive cells with the same attributes into spans
  const spans = useMemo(() => {
    const result: { cells: CellData[]; startCol: number }[] = [];
    let currentSpan: CellData[] = [];
    let spanStart = 0;

    for (let i = 0; i < cells.length; i++) {
      const cell = cells[i];
      if (currentSpan.length > 0 && !sameStyle(currentSpan[0], cell)) {
        result.push({ cells: currentSpan, startCol: spanStart });
        currentSpan = [];
        spanStart = i;
      }
      currentSpan.push(cell);
    }
    if (currentSpan.length > 0) {
      result.push({ cells: currentSpan, startCol: spanStart });
    }
    return result;
  }, [cells]);

  return (
    <div
      style={{
        position: "absolute",
        top,
        left: 0,
        right: 0,
        height: ROW_HEIGHT,
      }}
    >
      {spans.map((span, idx) => {
        const style = cellStyle(span.cells[0]);
        const text = span.cells.map((c) => c.content || " ").join("");

        // Check if cursor is within this span
        if (showCursor) {
          const cursorOffset = cursorCol - span.startCol;
          if (cursorOffset >= 0 && cursorOffset < span.cells.length) {
            const before = text.slice(0, cursorOffset);
            const cursorChar = text[cursorOffset] || " ";
            const after = text.slice(cursorOffset + 1);

            return (
              <React.Fragment key={idx}>
                {before && <span style={style}>{before}</span>}
                <span
                  style={cursorStyleFn(cursorShape)}
                  className="terminal-cursor"
                  data-cursor="true"
                >
                  {cursorChar}
                </span>
                {after && <span style={style}>{after}</span>}
              </React.Fragment>
            );
          }
        }

        return (
          <span key={idx} style={style}>
            {text}
          </span>
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
    colorEqual(a.fg, b.fg) &&
    colorEqual(a.bg, b.bg)
  );
}

function colorEqual(a: CellData["fg"], b: CellData["fg"]): boolean {
  if (a.type !== b.type) return false;
  if (a.type === "Default") return true;
  if (a.type === "Named" && b.type === "Named") return a.name === b.name;
  if (a.type === "Indexed" && b.type === "Indexed") return a.index === b.index;
  if (a.type === "Rgb" && b.type === "Rgb")
    return a.r === b.r && a.g === b.g && a.b === b.b;
  return false;
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

  const decorations: string[] = [];
  if (cell.underline) decorations.push("underline");
  if (cell.strikethrough) decorations.push("line-through");
  if (decorations.length > 0) style.textDecoration = decorations.join(" ");

  return style;
}

function cursorStyleFn(shape: string): React.CSSProperties {
  const cursorColor = "var(--term-cursor)";
  const bgColor = "var(--term-bg)";

  if (shape === "Underline") {
    return {
      borderBottom: `2px solid ${cursorColor}`,
    };
  }
  if (shape === "Bar") {
    return {
      borderLeft: `2px solid ${cursorColor}`,
    };
  }

  return {
    backgroundColor: cursorColor,
    color: bgColor,
  };
}
