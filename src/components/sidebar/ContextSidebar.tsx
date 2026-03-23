import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RotateCw, ChevronDown } from "lucide-react";
import { useSessionStore } from "../../stores/sessionStore";


interface ContextInfo {
  provider: string;
  detected: boolean;
  [key: string]: unknown;
}

interface ProjectContext {
  cwd: string;
  providers: ContextInfo[];
}

export function ContextSidebar() {
  const [context, setContext] = useState<ProjectContext | null>(null);
  const activeSessionId = useSessionStore((s) => s.activeSessionId);
  const sessions = useSessionStore((s) => s.sessions);
  const activeSession = sessions.find((s) => s.id === activeSessionId);
  const cwd = activeSession?.cwd || "";

  const refreshContext = useCallback(() => {
    if (!cwd) return;
    invoke<ProjectContext>("get_context", { cwd })
      .then(setContext)
      .catch(() => setContext(null));
  }, [cwd]);

  useEffect(() => {
    refreshContext();
  }, [refreshContext, cwd]);

  return (
    <aside
      role="complementary"
      className="flex flex-col select-none shrink-0 border-l border-[var(--color-border-muted)] bg-[var(--color-surface)]/60 backdrop-blur-xl"
      style={{ width: 280 }}
    >
      {/* Header - 48px */}
      <div
        className="flex items-center justify-between shrink-0 h-12 px-4 border-b border-[var(--color-border-muted)]"
      >
        <span className="text-sm font-bold text-[var(--color-text)]">
          Context
        </span>
        <button
          onClick={refreshContext}
          className="w-8 h-8 flex items-center justify-center rounded-lg text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-surface-hover)] transition-all"
          title="Refresh"
        >
          <RotateCw size={14} strokeWidth={2} />
        </button>
      </div>

      {/* Sections */}
      <div className="flex-1 overflow-y-auto custom-scrollbar">
        {context?.providers.map((provider) => (
          <ProviderSection key={provider.provider} provider={provider} />
        ))}

        {!context?.providers.length && cwd && (
          <div className="py-20 px-4 text-center text-xs text-[var(--color-text-muted)]">
            No project detected
          </div>
        )}
      </div>
    </aside>

  );
}

const PROVIDER_META: Record<string, { label: string; color: string }> = {
  git: { label: "Git", color: "var(--color-error)" },
  node: { label: "Node.js", color: "var(--color-success)" },
  rust: { label: "Rust", color: "var(--color-accent)" },
  python: { label: "Python", color: "var(--color-info)" },
  docker: { label: "Docker", color: "var(--color-primary)" },
};

function ProviderSection({ provider }: { provider: ContextInfo }) {
  const [collapsed, setCollapsed] = useState(false);
  const meta = PROVIDER_META[provider.provider] || {
    label: provider.provider,
    color: "var(--color-text-muted)",
  };

  const dataEntries = Object.entries(provider).filter(
    ([k]) => k !== "provider" && k !== "detected",
  );

  return (
    <div style={{ borderBottom: "1px solid var(--color-border-muted)" }}>
      {/* Section header - 32px */}
      <button
        onClick={() => setCollapsed(!collapsed)}
        style={{ height: 32, padding: "0 var(--sp-3)" }}
        className="w-full flex items-center gap-2 hover:bg-[var(--color-surface-hover)] transition-colors"
      >
        <ChevronDown
          size={12}
          strokeWidth={2}
          style={{
            transform: collapsed ? "rotate(-90deg)" : "rotate(0)",
            transition: "var(--transition-fast)",
            color: "var(--color-text-muted)",
          }}
        />
        <span
          style={{
            width: 8,
            height: 8,
            borderRadius: "50%",
            background: meta.color,
          }}
          className="shrink-0"
        />
        <span
          style={{
            fontSize: 12,
            fontWeight: 500,
            color: "var(--color-text)",
          }}
          className="flex-1 text-left"
        >
          {meta.label}
        </span>
      </button>

      {/* Section content */}
      {!collapsed && (
        <div
          style={{
            padding: "var(--sp-2) var(--sp-3)",
            paddingLeft: "calc(var(--sp-3) + 20px)",
          }}
        >
          {dataEntries.map(([key, value]) => (
            <div
              key={key}
              className="flex items-baseline gap-2"
              style={{ marginBottom: "var(--sp-1)" }}
            >
              <span
                style={{
                  fontSize: 11,
                  color: "var(--color-text-muted)",
                  minWidth: 60,
                }}
                className="shrink-0"
              >
                {key}
              </span>
              <span
                style={{
                  fontSize: 12,
                  color: "var(--color-text-secondary)",
                  fontFamily: key === "branch" ? "var(--font-mono)" : "inherit",
                }}
                className="truncate"
                title={String(value)}
              >
                {formatValue(key, value)}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function formatValue(key: string, value: unknown): string {
  const str = String(value);
  if (key === "status") {
    return str === "clean" ? "Clean" : str === "dirty" ? "Dirty" : str;
  }
  if (key === "scripts") {
    return str.split(",").slice(0, 5).join(", ");
  }
  return str;
}
