import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTheme } from "../../hooks/useTheme";

interface AppConfig {
  font_family: string;
  font_size: number;
  theme: string;
  cursor_style: string;
  cursor_blink: boolean;
  scrollback_size: number;
  sidebar_visible: boolean;
}

interface SettingsModalProps {
  visible: boolean;
  onClose: () => void;
}

export function SettingsModal({ visible, onClose }: SettingsModalProps) {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [activeTab, setActiveTab] = useState<"general" | "appearance" | "terminal">("general");
  const { themes, switchTheme } = useTheme();

  useEffect(() => {
    if (visible) {
      invoke<AppConfig>("get_config").then(setConfig).catch(() => {});
    }
  }, [visible]);

  const updateConfig = useCallback(
    (patch: Partial<AppConfig>) => {
      if (!config) return;
      const updated = { ...config, ...patch };
      setConfig(updated);
      invoke("set_config", { config: updated }).catch(() => {});

      // Apply theme change immediately
      if (patch.theme) {
        switchTheme(patch.theme);
      }

      // Apply font changes immediately
      if (patch.font_family || patch.font_size) {
        const root = document.documentElement;
        if (patch.font_family) {
          root.style.setProperty("--font-mono", patch.font_family);
        }
        if (patch.font_size) {
          root.style.fontSize = `${patch.font_size}px`;
        }
      }
    },
    [config, switchTheme],
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        onClose();
      }
    },
    [onClose],
  );

  if (!visible || !config) return null;

  const tabs = [
    { id: "general" as const, label: "General" },
    { id: "appearance" as const, label: "Appearance" },
    { id: "terminal" as const, label: "Terminal" },
  ];

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      onClick={onClose}
      onKeyDown={handleKeyDown}
    >
      <div
        className="w-[600px] max-h-[80vh] bg-[var(--ui-bg)] border border-[var(--ui-border)] rounded-lg shadow-2xl flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-3 border-b border-[var(--ui-border)]">
          <h2 className="text-base font-semibold text-[var(--ui-fg)]">Settings</h2>
          <button
            onClick={onClose}
            className="text-[var(--ui-fg-dim)] hover:text-[var(--term-red)] text-lg"
          >
            x
          </button>
        </div>

        {/* Tab bar */}
        <div className="flex border-b border-[var(--ui-border)]">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`px-4 py-2 text-sm ${
                activeTab === tab.id
                  ? "text-[var(--ui-accent)] border-b-2 border-[var(--ui-accent)]"
                  : "text-[var(--ui-fg-dim)] hover:text-[var(--ui-fg)]"
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-5 space-y-5">
          {activeTab === "general" && (
            <>
              <SettingRow label="Sidebar visible">
                <Toggle
                  value={config.sidebar_visible}
                  onChange={(v) => updateConfig({ sidebar_visible: v })}
                />
              </SettingRow>
            </>
          )}

          {activeTab === "appearance" && (
            <>
              <SettingRow label="Theme">
                <select
                  value={config.theme}
                  onChange={(e) => updateConfig({ theme: e.target.value })}
                  className="bg-[var(--ui-bg-tertiary)] text-[var(--ui-fg)] border border-[var(--ui-border)] rounded px-2 py-1 text-sm"
                >
                  {themes.map((t) => (
                    <option key={t} value={t}>
                      {t}
                    </option>
                  ))}
                </select>
              </SettingRow>

              <SettingRow label="Font family">
                <input
                  type="text"
                  value={config.font_family}
                  onChange={(e) => updateConfig({ font_family: e.target.value })}
                  className="bg-[var(--ui-bg-tertiary)] text-[var(--ui-fg)] border border-[var(--ui-border)] rounded px-2 py-1 text-sm w-64"
                />
              </SettingRow>

              <SettingRow label="Font size">
                <div className="flex items-center gap-2">
                  <input
                    type="range"
                    min={8}
                    max={24}
                    step={1}
                    value={config.font_size}
                    onChange={(e) =>
                      updateConfig({ font_size: parseFloat(e.target.value) })
                    }
                    className="w-32"
                  />
                  <span className="text-sm text-[var(--ui-fg-muted)] w-8">
                    {config.font_size}
                  </span>
                </div>
              </SettingRow>
            </>
          )}

          {activeTab === "terminal" && (
            <>
              <SettingRow label="Cursor style">
                <select
                  value={config.cursor_style}
                  onChange={(e) => updateConfig({ cursor_style: e.target.value })}
                  className="bg-[var(--ui-bg-tertiary)] text-[var(--ui-fg)] border border-[var(--ui-border)] rounded px-2 py-1 text-sm"
                >
                  <option value="block">Block</option>
                  <option value="underline">Underline</option>
                  <option value="bar">Bar</option>
                </select>
              </SettingRow>

              <SettingRow label="Cursor blink">
                <Toggle
                  value={config.cursor_blink}
                  onChange={(v) => updateConfig({ cursor_blink: v })}
                />
              </SettingRow>

              <SettingRow label="Scrollback lines">
                <input
                  type="number"
                  min={1000}
                  max={100000}
                  step={1000}
                  value={config.scrollback_size}
                  onChange={(e) =>
                    updateConfig({
                      scrollback_size: parseInt(e.target.value) || 10000,
                    })
                  }
                  className="bg-[var(--ui-bg-tertiary)] text-[var(--ui-fg)] border border-[var(--ui-border)] rounded px-2 py-1 text-sm w-24"
                />
              </SettingRow>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

function SettingRow({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between">
      <label className="text-sm text-[var(--ui-fg)]">{label}</label>
      {children}
    </div>
  );
}

function Toggle({
  value,
  onChange,
}: {
  value: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <button
      onClick={() => onChange(!value)}
      className={`w-10 h-5 rounded-full transition-colors relative ${
        value ? "bg-[var(--ui-accent)]" : "bg-[var(--ui-border)]"
      }`}
    >
      <span
        className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform ${
          value ? "translate-x-5" : "translate-x-0.5"
        }`}
      />
    </button>
  );
}
