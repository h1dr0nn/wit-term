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

  root.style.setProperty("--term-fg", colors.foreground);
  root.style.setProperty("--term-bg", colors.background);
  root.style.setProperty("--term-cursor", colors.cursor);
  root.style.setProperty("--term-selection-bg", colors.selection_bg);
  root.style.setProperty("--term-selection-fg", colors.selection_fg);

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

  // Derived UI colors
  root.style.setProperty("--ui-bg", colors.background);
  root.style.setProperty("--ui-bg-secondary", mixColor(colors.background, colors.foreground, 0.05));
  root.style.setProperty("--ui-bg-tertiary", mixColor(colors.background, colors.foreground, 0.1));
  root.style.setProperty("--ui-border", mixColor(colors.background, colors.foreground, 0.15));
  root.style.setProperty("--ui-fg", colors.foreground);
  root.style.setProperty("--ui-fg-muted", mixColor(colors.foreground, colors.background, 0.4));
  root.style.setProperty("--ui-fg-dim", mixColor(colors.foreground, colors.background, 0.6));
  root.style.setProperty("--ui-accent", colors.blue);
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
