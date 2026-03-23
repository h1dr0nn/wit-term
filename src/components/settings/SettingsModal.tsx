import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { X, Monitor, Palette, Terminal as TerminalIcon } from "lucide-react";
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

      if (patch.theme) {
        switchTheme(patch.theme);
      }

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
    { id: "general" as const, label: "General", icon: Monitor },
    { id: "appearance" as const, label: "Appearance", icon: Palette },
    { id: "terminal" as const, label: "Terminal", icon: TerminalIcon },
  ];

  return (
    <div
      className="modal-overlay"
      onClick={onClose}
      onKeyDown={handleKeyDown}
    >
      <div
        className="modal-content w-[720px] h-[520px] glass border border-[var(--color-border)] rounded-xl shadow-2xl flex overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Sidebar */}
        <div className="w-56 bg-[var(--color-surface)]/50 border-r border-[var(--color-border)] flex flex-col p-4">
          <div className="px-3 py-4 mb-2">
            <h2 className="text-lg font-bold tracking-tight text-[var(--color-text)]">Settings</h2>
          </div>
          
          <nav className="flex-1 space-y-1">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-all ${
                  activeTab === tab.id
                    ? "bg-[var(--color-primary-muted)] text-[var(--color-primary)] shadow-sm"
                    : "text-[var(--color-text-secondary)] hover:text-[var(--color-text)] hover:bg-[var(--color-surface-hover)]"
                }`}
              >
                <tab.icon size={18} strokeWidth={activeTab === tab.id ? 2.5 : 2} />
                {tab.label}
              </button>
            ))}
          </nav>

          <div className="mt-auto px-3 py-4 border-t border-[var(--color-border-muted)]">
            <div className="text-[10px] uppercase tracking-widest text-[var(--color-text-muted)] font-bold">
              Wit-Term v0.1.0
            </div>
          </div>
        </div>

        {/* Content Area */}
        <div className="flex-1 flex flex-col bg-[var(--color-surface)]/30 backdrop-blur-md">
          {/* Content Header */}
          <div className="h-14 flex items-center justify-between px-8 border-b border-[var(--color-border-muted)] shrink-0">
            <span className="text-sm font-bold uppercase tracking-wider text-[var(--color-text-secondary)]">
              {activeTab}
            </span>
            <button
              onClick={onClose}
              className="w-8 h-8 flex items-center justify-center rounded-lg text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-surface-hover)] transition-all"
            >
              <X size={18} />
            </button>
          </div>

          {/* Scrolling Content */}
          <div className="flex-1 overflow-y-auto p-8 custom-scrollbar">
            <div className="max-w-2xl mx-auto space-y-8">
              {activeTab === "general" && (
                <section className="space-y-6">
                  <SettingGroup title="Basic Layout">
                    <SettingRow label="Sidebar Visible" description="Show or hide the session sidebar by default.">
                      <Toggle
                        value={config.sidebar_visible}
                        onChange={(v) => updateConfig({ sidebar_visible: v })}
                      />
                    </SettingRow>
                  </SettingGroup>
                </section>
              )}

              {activeTab === "appearance" && (
                <section className="space-y-6">
                  <SettingGroup title="Themes & UI">
                    <SettingRow label="Interface Theme" description="Choose your preferred color palette.">
                      <select
                        value={config.theme}
                        onChange={(e) => updateConfig({ theme: e.target.value })}
                        className="bg-[var(--color-surface-hover)] text-[var(--color-text)] border border-[var(--color-border)] rounded-lg px-3 py-1.5 text-sm outline-none focus:border-[var(--color-primary)] transition-colors min-w-[140px]"
                      >
                        {themes.map((t) => (
                          <option key={t.id} value={t.id}>{t.name}</option>
                        ))}

                      </select>
                    </SettingRow>
                  </SettingGroup>

                  <SettingGroup title="Typography">
                    <SettingRow label="Font Family" description="The monospace font used in the terminal.">
                      <input
                        type="text"
                        value={config.font_family}
                        onChange={(e) => updateConfig({ font_family: e.target.value })}
                        className="bg-[var(--color-surface-hover)] text-[var(--color-text)] border border-[var(--color-border)] rounded-lg px-3 py-1.5 text-sm outline-none focus:border-[var(--color-primary)] transition-colors w-full max-w-[200px]"
                      />
                    </SettingRow>

                    <SettingRow label="Font Size" description={`${config.font_size}px`}>
                      <input
                        type="range"
                        min={8}
                        max={24}
                        step={1}
                        value={config.font_size}
                        onChange={(e) => updateConfig({ font_size: parseFloat(e.target.value) })}
                        className="w-32 accent-[var(--color-primary)]"
                      />
                    </SettingRow>
                  </SettingGroup>
                </section>
              )}

              {activeTab === "terminal" && (
                <section className="space-y-6">
                  <SettingGroup title="Cursor & Interaction">
                    <SettingRow label="Cursor Style" description="Visual appearance of the cursor.">
                      <select
                        value={config.cursor_style}
                        onChange={(e) => updateConfig({ cursor_style: e.target.value })}
                        className="bg-[var(--color-surface-hover)] text-[var(--color-text)] border border-[var(--color-border)] rounded-lg px-3 py-1.5 text-sm outline-none focus:border-[var(--color-primary)] transition-colors min-w-[120px]"
                      >
                        <option value="block">Block</option>
                        <option value="underline">Underline</option>
                        <option value="bar">Bar</option>
                      </select>
                    </SettingRow>

                    <SettingRow label="Cursor Blinking" description="Enable smooth cursor animation.">
                      <Toggle
                        value={config.cursor_blink}
                        onChange={(v) => updateConfig({ cursor_blink: v })}
                      />
                    </SettingRow>
                  </SettingGroup>

                  <SettingGroup title="Scrollback Buffer">
                    <SettingRow label="Maximum Lines" description="Number of lines kept in history.">
                      <input
                        type="number"
                        min={1000}
                        max={100000}
                        step={1000}
                        value={config.scrollback_size}
                        onChange={(e) => updateConfig({ scrollback_size: parseInt(e.target.value) || 10000 })}
                        className="bg-[var(--color-surface-hover)] text-[var(--color-text)] border border-[var(--color-border)] rounded-lg px-3 py-1.5 text-sm outline-none focus:border-[var(--color-primary)] transition-colors w-24"
                      />
                    </SettingRow>
                  </SettingGroup>
                </section>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function SettingGroup({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="space-y-4">
      <h3 className="text-xs font-bold uppercase tracking-widest text-[var(--color-text-muted)] px-1">
        {title}
      </h3>
      <div className="space-y-4 bg-[var(--color-surface-active)]/20 rounded-xl p-4 border border-[var(--color-border-muted)]">
        {children}
      </div>
    </div>
  );
}

function SettingRow({
  label,
  description,
  children,
}: {
  label: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between gap-8">
      <div className="flex-1 min-w-0">
        <label className="block text-sm font-medium text-[var(--color-text)] mb-0.5">{label}</label>
        {description && (
          <p className="text-xs text-[var(--color-text-secondary)] leading-normal">{description}</p>
        )}
      </div>
      <div className="shrink-0">
        {children}
      </div>
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
      className={`w-10 h-6 rounded-full transition-all duration-300 relative group p-1 ${
        value 
          ? "bg-[var(--color-primary)]" 
          : "bg-[var(--color-border)] hover:bg-[var(--color-surface-active)]"
      }`}
    >
      <span
        className={`block w-4 h-4 rounded-full bg-white shadow-sm transition-all duration-300 ease-[cubic-bezier(0.34,1.56,0.64,1)] ${
          value ? "translate-x-4 scale-110" : "translate-x-0"
        }`}
      />
    </button>
  );
}

