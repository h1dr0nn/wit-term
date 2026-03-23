import type { ColorData } from "../stores/terminalStore";

// Standard ANSI 16-color palette (Catppuccin Mocha-inspired)
const NAMED_COLORS: Record<string, string> = {
  Black: "#45475a",
  Red: "#f38ba8",
  Green: "#a6e3a1",
  Yellow: "#f9e2af",
  Blue: "#89b4fa",
  Magenta: "#f5c2e7",
  Cyan: "#94e2d5",
  White: "#bac2de",
  BrightBlack: "#585b70",
  BrightRed: "#f38ba8",
  BrightGreen: "#a6e3a1",
  BrightYellow: "#f9e2af",
  BrightBlue: "#89b4fa",
  BrightMagenta: "#f5c2e7",
  BrightCyan: "#94e2d5",
  BrightWhite: "#cdd6f4",
};

// 256-color lookup table (first 16 are ANSI, 16-231 are 6x6x6 cube, 232-255 are grayscale)
function indexed256(index: number): string {
  if (index < 16) {
    const names = [
      "Black",
      "Red",
      "Green",
      "Yellow",
      "Blue",
      "Magenta",
      "Cyan",
      "White",
      "BrightBlack",
      "BrightRed",
      "BrightGreen",
      "BrightYellow",
      "BrightBlue",
      "BrightMagenta",
      "BrightCyan",
      "BrightWhite",
    ];
    return NAMED_COLORS[names[index]] || "#cdd6f4";
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
      return isFg ? "#cdd6f4" : undefined;
    case "Named":
      return NAMED_COLORS[color.name] || (isFg ? "#cdd6f4" : undefined);
    case "Indexed":
      return indexed256(color.index);
    case "Rgb":
      return `rgb(${color.r},${color.g},${color.b})`;
  }
}
