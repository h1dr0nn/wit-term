import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface AppConfig {
  font_family: string;
  font_size: number;
  theme: string;
  cursor_style: string;
  cursor_blink: boolean;
  scrollback_size: number;
  sidebar_visible: boolean;
}

const DEFAULT_CONFIG: AppConfig = {
  font_family: "monospace",
  font_size: 14,
  theme: "wit-dark",
  cursor_style: "block",
  cursor_blink: true,
  scrollback_size: 10000,
  sidebar_visible: true,
};

interface SettingsState {
  config: AppConfig;
  loaded: boolean;

  loadSettings: () => Promise<void>;
  updateSetting: <K extends keyof AppConfig>(key: K, value: AppConfig[K]) => Promise<void>;
  updateSettings: (patch: Partial<AppConfig>) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  config: DEFAULT_CONFIG,
  loaded: false,

  loadSettings: async () => {
    try {
      const config = await invoke<AppConfig>("get_config");
      set({ config, loaded: true });
    } catch {
      set({ loaded: true });
    }
  },

  updateSetting: async (key, value) => {
    const updated = { ...get().config, [key]: value };
    set({ config: updated });
    try {
      await invoke("set_config", { config: updated });
    } catch {
      // Revert on failure
    }
  },

  updateSettings: async (patch) => {
    const updated = { ...get().config, ...patch };
    set({ config: updated });
    try {
      await invoke("set_config", { config: updated });
    } catch {
      // Revert on failure
    }
  },
}));
