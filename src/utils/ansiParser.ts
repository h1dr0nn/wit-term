/** Parse text with ANSI SGR codes into styled spans. */
export interface AnsiSpan {
  text: string;
  bold?: boolean;
  dim?: boolean;
  italic?: boolean;
  underline?: boolean;
  strikethrough?: boolean;
  fg?: string;
  bg?: string;
}

// Standard 8-color palette
const COLORS_16: Record<number, string> = {
  30: "var(--term-black, #1d1f21)",
  31: "var(--term-red, #cc6666)",
  32: "var(--term-green, #b5bd68)",
  33: "var(--term-yellow, #f0c674)",
  34: "var(--term-blue, #81a2be)",
  35: "var(--term-magenta, #b294bb)",
  36: "var(--term-cyan, #8abeb7)",
  37: "var(--term-white, #c5c8c6)",
  90: "var(--term-bright-black, #969896)",
  91: "var(--term-bright-red, #de935f)",
  92: "var(--term-bright-green, #b5bd68)",
  93: "var(--term-bright-yellow, #f0c674)",
  94: "var(--term-bright-blue, #81a2be)",
  95: "var(--term-bright-magenta, #b294bb)",
  96: "var(--term-bright-cyan, #8abeb7)",
  97: "var(--term-bright-white, #ffffff)",
};

const BG_COLORS_16: Record<number, string> = {
  40: "var(--term-black, #1d1f21)",
  41: "var(--term-red, #cc6666)",
  42: "var(--term-green, #b5bd68)",
  43: "var(--term-yellow, #f0c674)",
  44: "var(--term-blue, #81a2be)",
  45: "var(--term-magenta, #b294bb)",
  46: "var(--term-cyan, #8abeb7)",
  47: "var(--term-white, #c5c8c6)",
  100: "var(--term-bright-black, #969896)",
  101: "var(--term-bright-red, #de935f)",
  102: "var(--term-bright-green, #b5bd68)",
  103: "var(--term-bright-yellow, #f0c674)",
  104: "var(--term-bright-blue, #81a2be)",
  105: "var(--term-bright-magenta, #b294bb)",
  106: "var(--term-bright-cyan, #8abeb7)",
  107: "var(--term-bright-white, #ffffff)",
};

// 256-color palette (indexed 0-255)
function color256(n: number): string | undefined {
  if (n < 8) return COLORS_16[30 + n];
  if (n < 16) return COLORS_16[90 + n - 8];
  if (n < 232) {
    // 216-color cube: 16 + 36*r + 6*g + b
    const idx = n - 16;
    const r = Math.floor(idx / 36);
    const g = Math.floor((idx % 36) / 6);
    const b = idx % 6;
    const toHex = (v: number) => (v === 0 ? 0 : 55 + v * 40);
    return `rgb(${toHex(r)},${toHex(g)},${toHex(b)})`;
  }
  if (n < 256) {
    // Grayscale: 232-255
    const v = 8 + (n - 232) * 10;
    return `rgb(${v},${v},${v})`;
  }
  return undefined;
}

interface SgrState {
  bold: boolean;
  dim: boolean;
  italic: boolean;
  underline: boolean;
  strikethrough: boolean;
  fg?: string;
  bg?: string;
}

function applySgr(state: SgrState, params: number[]): void {
  let i = 0;
  while (i < params.length) {
    const p = params[i];
    if (p === 0) {
      state.bold = false;
      state.dim = false;
      state.italic = false;
      state.underline = false;
      state.strikethrough = false;
      state.fg = undefined;
      state.bg = undefined;
    } else if (p === 1) {
      state.bold = true;
    } else if (p === 2) {
      state.dim = true;
    } else if (p === 3) {
      state.italic = true;
    } else if (p === 4) {
      state.underline = true;
    } else if (p === 9) {
      state.strikethrough = true;
    } else if (p === 22) {
      state.bold = false;
      state.dim = false;
    } else if (p === 23) {
      state.italic = false;
    } else if (p === 24) {
      state.underline = false;
    } else if (p === 29) {
      state.strikethrough = false;
    } else if (p >= 30 && p <= 37) {
      state.fg = COLORS_16[p];
    } else if (p === 38) {
      // Extended foreground
      if (params[i + 1] === 5 && i + 2 < params.length) {
        state.fg = color256(params[i + 2]);
        i += 2;
      } else if (params[i + 1] === 2 && i + 4 < params.length) {
        state.fg = `rgb(${params[i + 2]},${params[i + 3]},${params[i + 4]})`;
        i += 4;
      }
    } else if (p === 39) {
      state.fg = undefined;
    } else if (p >= 40 && p <= 47) {
      state.bg = BG_COLORS_16[p];
    } else if (p === 48) {
      // Extended background
      if (params[i + 1] === 5 && i + 2 < params.length) {
        state.bg = color256(params[i + 2]);
        i += 2;
      } else if (params[i + 1] === 2 && i + 4 < params.length) {
        state.bg = `rgb(${params[i + 2]},${params[i + 3]},${params[i + 4]})`;
        i += 4;
      }
    } else if (p === 49) {
      state.bg = undefined;
    } else if (p >= 90 && p <= 97) {
      state.fg = COLORS_16[p];
    } else if (p >= 100 && p <= 107) {
      state.bg = BG_COLORS_16[p];
    }
    i++;
  }
}

/** Parse a string containing ANSI SGR codes into styled spans. */
export function parseAnsi(input: string): AnsiSpan[] {
  const spans: AnsiSpan[] = [];
  const state: SgrState = {
    bold: false,
    dim: false,
    italic: false,
    underline: false,
    strikethrough: false,
  };

  // Regex to match CSI SGR sequences: \x1b[ params m
  const re = /\x1b\[([\d;]*)m/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = re.exec(input)) !== null) {
    // Text before this escape
    if (match.index > lastIndex) {
      const text = input.slice(lastIndex, match.index);
      if (text) {
        spans.push({
          text,
          ...(state.bold && { bold: true }),
          ...(state.dim && { dim: true }),
          ...(state.italic && { italic: true }),
          ...(state.underline && { underline: true }),
          ...(state.strikethrough && { strikethrough: true }),
          ...(state.fg && { fg: state.fg }),
          ...(state.bg && { bg: state.bg }),
        });
      }
    }

    // Apply SGR params
    const paramStr = match[1];
    const params = paramStr ? paramStr.split(";").map(Number) : [0];
    applySgr(state, params);
    lastIndex = re.lastIndex;
  }

  // Remaining text after last escape
  if (lastIndex < input.length) {
    const text = input.slice(lastIndex);
    if (text) {
      spans.push({
        text,
        ...(state.bold && { bold: true }),
        ...(state.dim && { dim: true }),
        ...(state.italic && { italic: true }),
        ...(state.underline && { underline: true }),
        ...(state.strikethrough && { strikethrough: true }),
        ...(state.fg && { fg: state.fg }),
        ...(state.bg && { bg: state.bg }),
      });
    }
  }

  return spans;
}

/** Convert AnsiSpan to CSS properties. */
export function spanStyle(span: AnsiSpan): React.CSSProperties {
  const style: React.CSSProperties = {};
  if (span.fg) style.color = span.fg;
  if (span.bg) style.backgroundColor = span.bg;
  if (span.bold) style.fontWeight = "bold";
  if (span.dim) style.opacity = 0.5;
  if (span.italic) style.fontStyle = "italic";
  const deco: string[] = [];
  if (span.underline) deco.push("underline");
  if (span.strikethrough) deco.push("line-through");
  if (deco.length) style.textDecoration = deco.join(" ");
  return style;
}
