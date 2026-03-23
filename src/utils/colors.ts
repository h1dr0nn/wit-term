import type { ColorData } from "../stores/terminalStore";

// Map ANSI named colors to CSS custom properties
const NAMED_COLOR_VARS: Record<string, string> = {
  Black: "var(--term-black)",
  Red: "var(--term-red)",
  Green: "var(--term-green)",
  Yellow: "var(--term-yellow)",
  Blue: "var(--term-blue)",
  Magenta: "var(--term-magenta)",
  Cyan: "var(--term-cyan)",
  White: "var(--term-white)",
  BrightBlack: "var(--term-bright-black)",
  BrightRed: "var(--term-bright-red)",
  BrightGreen: "var(--term-bright-green)",
  BrightYellow: "var(--term-bright-yellow)",
  BrightBlue: "var(--term-bright-blue)",
  BrightMagenta: "var(--term-bright-magenta)",
  BrightCyan: "var(--term-bright-cyan)",
  BrightWhite: "var(--term-bright-white)",
};

// Indexed color names for the first 16 colors
const INDEXED_NAMES = [
  "Black", "Red", "Green", "Yellow", "Blue", "Magenta", "Cyan", "White",
  "BrightBlack", "BrightRed", "BrightGreen", "BrightYellow",
  "BrightBlue", "BrightMagenta", "BrightCyan", "BrightWhite",
];

// 256-color lookup (16-255)
function indexed256(index: number): string {
  if (index < 16) {
    return NAMED_COLOR_VARS[INDEXED_NAMES[index]] || "var(--term-fg)";
  }

  if (index < 232) {
    // 6x6x6 color cube
    const i = index - 16;
    const r = Math.floor(i / 36);
    const g = Math.floor((i % 36) / 6);
    const b = i % 6;
    const toVal = (v: number) => (v === 0 ? 0 : 55 + v * 40);
    return `rgb(${toVal(r)},${toVal(g)},${toVal(b)})`;
  }

  // Grayscale ramp
  const gray = 8 + (index - 232) * 10;
  return `rgb(${gray},${gray},${gray})`;
}

export function colorToCss(color: ColorData, isFg: boolean): string | undefined {
  switch (color.type) {
    case "Default":
      return isFg ? "var(--term-fg)" : undefined;
    case "Named":
      return NAMED_COLOR_VARS[color.name] || (isFg ? "var(--term-fg)" : undefined);
    case "Indexed":
      return indexed256(color.index);
    case "Rgb":
      return `rgb(${color.r},${color.g},${color.b})`;
  }
}
