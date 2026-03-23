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

  // Fetch context when cwd changes
  useEffect(() => {
    refreshContext();
  }, [refreshContext, cwd]);

  return (
    <aside className="w-56 bg-[var(--ui-bg-secondary)] border-l border-[var(--ui-border)] flex flex-col select-none overflow-y-auto">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-[var(--ui-border)]">
        <span className="text-xs font-semibold text-[var(--ui-fg-muted)] uppercase tracking-wider">
          Context
        </span>
        <button
          onClick={refreshContext}
          className="text-[var(--ui-fg-dim)] hover:text-[var(--ui-fg)] text-xs px-1"
          title="Refresh"
        >
          R
        </button>
      </div>

      {/* CWD */}
      {cwd && (
        <div className="px-3 py-2 border-b border-[var(--ui-border)]">
          <div className="text-xs text-[var(--ui-fg-dim)]">Directory</div>
          <div className="text-sm text-[var(--ui-fg)] truncate" title={cwd}>
            {cwdShort(cwd)}
          </div>
        </div>
      )}

      {/* Providers */}
      {context?.providers.map((provider) => (
        <ProviderSection key={provider.provider} provider={provider} />
      ))}

      {!context?.providers.length && cwd && (
        <div className="px-3 py-4 text-center text-[var(--ui-fg-dim)] text-xs">
          No project detected
        </div>
      )}
    </aside>
  );
}

function ProviderSection({ provider }: { provider: ContextInfo }) {
  const [collapsed, setCollapsed] = useState(false);

  const providerIcons: Record<string, string> = {
    git: "G",
    node: "N",
    rust: "R",
    python: "P",
    docker: "D",
  };

  const providerColors: Record<string, string> = {
    git: "text-[var(--term-red)]",
    node: "text-[var(--term-green)]",
    rust: "text-[var(--term-yellow)]",
    python: "text-[var(--term-blue)]",
    docker: "text-[var(--term-cyan)]",
  };

  // Extract data fields (exclude provider, detected)
  const dataEntries = Object.entries(provider).filter(
    ([k]) => k !== "provider" && k !== "detected",
  );

  return (
    <div className="border-b border-[var(--ui-border)]">
      <button
        onClick={() => setCollapsed(!collapsed)}
        className="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-[var(--ui-bg)]"
      >
        <span
          className={`text-xs font-bold w-4 ${providerColors[provider.provider] || "text-[var(--ui-fg-dim)]"}`}
        >
          {providerIcons[provider.provider] || "?"}
        </span>
        <span className="text-[var(--ui-fg)] capitalize flex-1 text-left">
          {provider.provider}
        </span>
        <span className="text-[var(--ui-fg-dim)] text-xs">
          {collapsed ? "+" : "-"}
        </span>
      </button>

      {!collapsed && (
        <div className="px-3 pb-2 space-y-1">
          {dataEntries.map(([key, value]) => (
            <div key={key} className="flex items-baseline gap-2">
              <span className="text-xs text-[var(--ui-fg-dim)] shrink-0">
                {key}
              </span>
              <span
                className="text-xs text-[var(--ui-fg-muted)] truncate"
                title={String(value)}
              >
                {String(value)}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function cwdShort(cwd: string): string {
  if (!cwd) return "";
  const normalized = cwd.replace(/\\/g, "/");
  const parts = normalized.split("/").filter(Boolean);
  if (parts.length <= 3) return normalized;
  return `.../${parts.slice(-3).join("/")}`;
}
