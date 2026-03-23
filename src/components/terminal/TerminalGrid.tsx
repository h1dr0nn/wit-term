import React from "react";
import type { CellData, GridSnapshot } from "../../stores/terminalStore";
import { colorToCss } from "../../utils/colors";

interface TerminalGridProps {
  snapshot: GridSnapshot;
}

export function TerminalGrid({ snapshot }: TerminalGridProps) {
  return (
    <div className="font-mono text-sm leading-[1.2] whitespace-pre">
      {snapshot.rows.map((row, rowIdx) => (
        <TerminalRow
          key={rowIdx}
          cells={row}
          rowIdx={rowIdx}
          cursorCol={snapshot.cursor_col}
          showCursor={snapshot.cursor_visible && rowIdx === snapshot.cursor_row}
          cursorShape={snapshot.cursor_shape}
        />
      ))}
    </div>
  );
}

interface TerminalRowProps {
  cells: CellData[];
  rowIdx: number;
  cursorCol: number;
  showCursor: boolean;
  cursorShape: string;
}

const TerminalRow = React.memo(function TerminalRow({
  cells,
  cursorCol,
  showCursor,
  cursorShape,
}: TerminalRowProps) {
  // Group consecutive cells with the same attributes into spans
  const spans: { cells: CellData[]; startCol: number }[] = [];
  let currentSpan: CellData[] = [];
  let spanStart = 0;

  for (let i = 0; i < cells.length; i++) {
    const cell = cells[i];
    if (currentSpan.length > 0 && !sameStyle(currentSpan[0], cell)) {
      spans.push({ cells: currentSpan, startCol: spanStart });
      currentSpan = [];
      spanStart = i;
    }
    currentSpan.push(cell);
  }
  if (currentSpan.length > 0) {
    spans.push({ cells: currentSpan, startCol: spanStart });
  }

  return (
    <div className="h-[1.2em]">
      {spans.map((span, idx) => {
        const style = cellStyle(span.cells[0]);
        const text = span.cells.map((c) => c.content || " ").join("");

        // Check if cursor is within this span
        if (showCursor) {
          const cursorOffset = cursorCol - span.startCol;
          if (cursorOffset >= 0 && cursorOffset < span.cells.length) {
            // Split the span around the cursor
            const before = text.slice(0, cursorOffset);
            const cursorChar = text[cursorOffset] || " ";
            const after = text.slice(cursorOffset + 1);

            return (
              <React.Fragment key={idx}>
                {before && <span style={style}>{before}</span>}
                <span style={cursorStyle(cursorShape)} data-cursor="true">
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

function colorEqual(
  a: CellData["fg"],
  b: CellData["fg"],
): boolean {
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

function cursorStyle(shape: string): React.CSSProperties {
  const base: React.CSSProperties = {
    backgroundColor: "#cdd6f4",
    color: "#1e1e2e",
  };

  if (shape === "Underline") {
    return {
      borderBottom: "2px solid #cdd6f4",
      backgroundColor: undefined,
      color: undefined,
    };
  }
  if (shape === "Bar") {
    return {
      borderLeft: "2px solid #cdd6f4",
      backgroundColor: undefined,
      color: undefined,
    };
  }

  return base; // Block
}
