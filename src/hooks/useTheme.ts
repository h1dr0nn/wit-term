import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface ThemeColors {
  foreground: string;
  background: string;
  cursor: string;
  selection_bg: string;
  selection_fg: string;
  black: string;
  red: string;
  green: string;
  yellow: string;
  blue: string;
  magenta: string;
  cyan: string;
  white: string;
  bright_black: string;
  bright_red: string;
  bright_green: string;
  bright_yellow: string;
  bright_blue: string;
  bright_magenta: string;
  bright_cyan: string;
  bright_white: string;
}

export interface Theme {
  name: string;
  author: string;
  colors: ThemeColors;
}

/** Apply theme colors as CSS custom properties on :root */
function applyTheme(colors: ThemeColors) {
  const root = document.documentElement;

  // Terminal basic colors
  root.style.setProperty("--term-fg", colors.foreground);
  root.style.setProperty("--term-bg", colors.background);
  root.style.setProperty("--term-cursor", colors.cursor);
  root.style.setProperty("--term-selection-bg", colors.selection_bg);
  root.style.setProperty("--term-selection-fg", colors.selection_fg);

  // Terminal ANSI colors
  root.style.setProperty("--term-black", colors.black);
  root.style.setProperty("--term-red", colors.red);
  root.style.setProperty("--term-green", colors.green);
  root.style.setProperty("--term-yellow", colors.yellow);
  root.style.setProperty("--term-blue", colors.blue);
  root.style.setProperty("--term-magenta", colors.magenta);
  root.style.setProperty("--term-cyan", colors.cyan);
  root.style.setProperty("--term-white", colors.white);

  root.style.setProperty("--term-bright-black", colors.bright_black);
  root.style.setProperty("--term-bright-red", colors.bright_red);
  root.style.setProperty("--term-bright-green", colors.bright_green);
  root.style.setProperty("--term-bright-yellow", colors.bright_yellow);
  root.style.setProperty("--term-bright-blue", colors.bright_blue);
  root.style.setProperty("--term-bright-magenta", colors.bright_magenta);
  root.style.setProperty("--term-bright-cyan", colors.bright_cyan);
  root.style.setProperty("--term-bright-white", colors.bright_white);

  // --- Dynamic UI Synchronization ---
  const isLight = getLuminance(colors.background) > 0.5;
  const mixFactor = isLight ? 0.08 : 0.05;

  root.style.setProperty("--color-bg", colors.background);
  root.style.setProperty("--color-text", colors.foreground);
  
  // Mix colors for hierarchy
  const secondaryFg = mixColor(colors.foreground, colors.background, 0.3);
  const mutedFg = mixColor(colors.foreground, colors.background, 0.5);
  
  root.style.setProperty("--color-text-secondary", secondaryFg);
  root.style.setProperty("--color-text-muted", mutedFg);

  const surface = mixColor(colors.background, colors.foreground, mixFactor);
  const surfaceHover = mixColor(colors.background, colors.foreground, mixFactor * 2);
  const surfaceActive = mixColor(colors.background, colors.foreground, mixFactor * 3);
  
  root.style.setProperty("--color-surface", surface);
  root.style.setProperty("--color-surface-hover", surfaceHover);
  root.style.setProperty("--color-surface-active", surfaceActive);

  root.style.setProperty("--color-border", mixColor(colors.background, colors.foreground, 0.15));
  root.style.setProperty("--color-border-muted", mixColor(colors.background, colors.foreground, 0.08));

  // Accent & Functional colors
  root.style.setProperty("--color-primary", colors.blue);
  root.style.setProperty("--color-primary-muted", mixColor(colors.blue, colors.background, 0.8));
  root.style.setProperty("--color-success", colors.green);
  root.style.setProperty("--color-warning", colors.yellow);
  root.style.setProperty("--color-error", colors.red);
  root.style.setProperty("--color-info", colors.cyan);
}

function getLuminance(hex: string): number {
  const h = hex.replace("#", "");
  const r = parseInt(h.substring(0, 2), 16) / 255;
  const g = parseInt(h.substring(2, 4), 16) / 255;
  const b = parseInt(h.substring(4, 6), 16) / 255;
  
  const a = [r, g, b].map(v => {
    return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
  });
  return a[0] * 0.2126 + a[1] * 0.7152 + a[2] * 0.0722;
}


function mixColor(c1: string, c2: string, ratio: number): string {
  const parse = (hex: string) => {
    const h = hex.replace("#", "");
    return [
      parseInt(h.substring(0, 2), 16),
      parseInt(h.substring(2, 4), 16),
      parseInt(h.substring(4, 6), 16),
    ];
  };
  const [r1, g1, b1] = parse(c1);
  const [r2, g2, b2] = parse(c2);
  const r = Math.round(r1 + (r2 - r1) * ratio);
  const g = Math.round(g1 + (g2 - g1) * ratio);
  const b = Math.round(b1 + (b2 - b1) * ratio);
  return `#${r.toString(16).padStart(2, "0")}${g.toString(16).padStart(2, "0")}${b.toString(16).padStart(2, "0")}`;
}

export function useTheme() {
  const [currentTheme, setCurrentTheme] = useState<string>("wit-dark");
  const [themes, setThemes] = useState<string[]>([]);

  // Load theme list and apply current theme on mount
  useEffect(() => {
    invoke<string[]>("list_themes").then(setThemes).catch(() => {});

    invoke<{ theme: string }>("get_config")
      .then((config) => {
        const themeName = config.theme || "wit-dark";
        setCurrentTheme(themeName);
        loadAndApplyTheme(themeName);
      })
      .catch(() => {
        loadAndApplyTheme("wit-dark");
      });
  }, []);

  // Listen for theme changes
  useEffect(() => {
    const unlisten = listen<string>("theme_changed", (event) => {
      loadAndApplyTheme(event.payload);
      setCurrentTheme(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const switchTheme = useCallback(async (name: string) => {
    await loadAndApplyTheme(name);
    setCurrentTheme(name);
    // Save to config
    try {
      const config = await invoke<Record<string, unknown>>("get_config");
      await invoke("set_config", { config: { ...config, theme: name } });
    } catch {
      // Config save failed
    }
  }, []);

  return { currentTheme, themes, switchTheme };
}

async function loadAndApplyTheme(name: string) {
  try {
    const theme = await invoke<Theme>("get_theme", { name });
    applyTheme(theme.colors);
  } catch {
    // Theme not found, keep current
  }
}
