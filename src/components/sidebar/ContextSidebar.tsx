import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
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
      style={{
        width: 280,
        background: "var(--color-surface)",
        borderLeft: "1px solid var(--color-border-muted)",
      }}
      className="flex flex-col select-none shrink-0 overflow-y-auto"
    >
      {/* Header - 48px */}
      <div
        style={{
          height: 48,
          padding: "0 var(--sp-3)",
          borderBottom: "1px solid var(--color-border-muted)",
        }}
        className="flex items-center justify-between shrink-0"
      >
        <span style={{ fontSize: 13, fontWeight: 600, color: "var(--color-text)" }}>
          Context
        </span>
        <button
          onClick={refreshContext}
          style={{
            width: 28,
            height: 28,
            borderRadius: "var(--radius-md)",
            color: "var(--color-text-muted)",
          }}
          className="flex items-center justify-center hover:text-[var(--color-text)] hover:bg-[var(--color-surface-hover)] transition-colors"
          title="Refresh"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
            <path d="M1 7a6 6 0 0111.2-3M13 7a6 6 0 01-11.2 3" />
            <polyline points="1,3 1,7 5,7" />
            <polyline points="13,11 13,7 9,7" />
          </svg>
        </button>
      </div>

      {/* Sections */}
      {context?.providers.map((provider) => (
        <ProviderSection key={provider.provider} provider={provider} />
      ))}

      {!context?.providers.length && cwd && (
        <div
          style={{
            padding: "var(--sp-8) var(--sp-3)",
            color: "var(--color-text-muted)",
            fontSize: 12,
            textAlign: "center",
          }}
        >
          No project detected
        </div>
      )}
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
        <svg
          width="12"
          height="12"
          viewBox="0 0 12 12"
          fill="none"
          stroke="var(--color-text-muted)"
          strokeWidth="1.5"
          style={{
            transform: collapsed ? "rotate(-90deg)" : "rotate(0)",
            transition: "var(--transition-fast)",
          }}
        >
          <polyline points="3,4 6,7 9,4" />
        </svg>
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
