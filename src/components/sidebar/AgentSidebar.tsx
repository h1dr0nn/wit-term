import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAgentStore } from "../../stores/agentStore";
import { AgentTimeline } from "./AgentTimeline";
import { AgentFiles } from "./AgentFiles";

function formatTokens(n: number): string {
  if (n < 1000) return String(n);
  if (n < 10000) return `${(n / 1000).toFixed(1)}k`;
  if (n < 1000000) return `${(n / 1000).toFixed(1)}k`;
  return `${(n / 1000000).toFixed(1)}M`;
}

function costColorClass(cost: number): string {
  if (cost > 5) return "text-red-400";
  if (cost > 1) return "text-orange-400";
  return "text-[var(--color-text-secondary)]";
}

interface AgentSidebarProps {
  sessionId: string;
}

export function AgentSidebar({ sessionId }: AgentSidebarProps) {
  const session = useAgentStore((s) => s.sessions[sessionId]);
  const activeTab = useAgentStore((s) => s.activeTab);
  const setActiveTab = useAgentStore((s) => s.setActiveTab);
  const [showSettings, setShowSettings] = useState(false);

  if (!session) {
    return (
      <aside
        role="complementary"
        className="flex flex-col select-none shrink-0 border-l border-[var(--color-border-muted)] bg-[var(--color-surface)]"
        style={{ width: 280 }}
      >
        <div className="flex items-center shrink-0 h-12 px-4 border-b border-[var(--color-border-muted)]">
          <span className="text-sm font-bold text-[var(--color-text)]">Agent</span>
        </div>
        <div className="py-20 px-4 text-center text-xs text-[var(--color-text-muted)]">
          No agent detected
        </div>
      </aside>
    );
  }

  const { identity, events, fileChanges, totalInputTokens, totalOutputTokens, totalCost, isThinking, isEnded } = session;
  const totalTokens = totalInputTokens + totalOutputTokens;

  return (
    <aside
      role="complementary"
      className="flex flex-col select-none shrink-0 border-l border-[var(--color-border-muted)] bg-[var(--color-surface)]"
      style={{ width: 280 }}
    >
      {/* Header - 48px */}
      <div className="flex items-center justify-between shrink-0 h-12 px-4 border-b border-[var(--color-border-muted)]">
        <div className="flex items-center gap-2 min-w-0">
          <div className="w-2 h-2 rounded-full bg-green-400 flex-shrink-0" />
          <span className="text-sm font-bold text-[var(--color-text)] truncate">
            {identity.name}
          </span>
          <span className="text-[10px] px-1.5 py-0.5 rounded bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] flex-shrink-0">
            {identity.kind}
          </span>
        </div>
        {/* Token/Cost + settings gear */}
        <div className="flex items-center gap-2 flex-shrink-0 ml-2">
          {totalTokens > 0 && (
            <span className="text-[10px] text-[var(--color-text-muted)]">
              {formatTokens(totalTokens)}
            </span>
          )}
          {totalCost > 0 && (
            <span className={`text-[10px] font-mono ${costColorClass(totalCost)}`}>
              ${totalCost.toFixed(2)}
            </span>
          )}
          <button
            onClick={() => setShowSettings((v) => !v)}
            title="Agent settings"
            className="text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
            style={{ background: "none", border: "none", cursor: "pointer", padding: 2, fontSize: 13, lineHeight: 1 }}
          >
            {showSettings ? "\u2715" : "\u2699"}
          </button>
        </div>
      </div>

      {/* Inline settings panel */}
      {showSettings && (
        <div className="px-4 py-3 border-b border-[var(--color-border-muted)] bg-[var(--color-surface-hover)] space-y-3">
          {/* Effort level */}
          <div>
            <div className="text-xs text-[var(--color-text)] mb-1.5">Effort</div>
            <div className="flex gap-1">
              {(["low", "medium", "high", "max"] as const).map((level) => (
                <button
                  key={level}
                  onClick={() => {
                    invoke("send_input", { sessionId, data: `/effort ${level}\n` }).catch(() => {});
                  }}
                  className="flex-1 text-[10px] py-1 rounded transition-colors border"
                  style={{
                    background: "var(--color-surface)",
                    borderColor: "var(--color-border)",
                    color: "var(--color-text)",
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.background = "var(--color-primary)";
                    e.currentTarget.style.color = "#fff";
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.background = "var(--color-surface)";
                    e.currentTarget.style.color = "var(--color-text)";
                  }}
                >
                  {level}
                </button>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Ended state */}
      {isEnded && (
        <div className="px-4 py-2 border-b border-[var(--color-border-muted)] bg-[var(--color-surface-hover)]">
          <div className="text-xs text-[var(--color-text-muted)]">
            Agent session ended
          </div>
          <div className="text-[10px] text-[var(--color-text-muted)] mt-1 flex gap-3">
            <span>{events.length} events</span>
            <span>{fileChanges.length} files</span>
            {totalCost > 0 && <span>${totalCost.toFixed(2)}</span>}
          </div>
        </div>
      )}

      {/* Tab bar */}
      <div className="flex shrink-0 border-b border-[var(--color-border-muted)]">
        <button
          onClick={() => setActiveTab("activity")}
          className={`flex-1 h-8 text-xs font-medium transition-colors ${
            activeTab === "activity"
              ? "text-[var(--color-text)] border-b-2 border-[var(--color-primary)]"
              : "text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
          }`}
        >
          Activity
          {events.length > 0 && (
            <span className="ml-1 text-[10px] text-[var(--color-text-muted)]">
              ({events.length})
            </span>
          )}
        </button>
        <button
          onClick={() => setActiveTab("files")}
          className={`flex-1 h-8 text-xs font-medium transition-colors ${
            activeTab === "files"
              ? "text-[var(--color-text)] border-b-2 border-[var(--color-primary)]"
              : "text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
          }`}
        >
          Files
          {fileChanges.length > 0 && (
            <span className="ml-1 text-[10px] text-[var(--color-text-muted)]">
              ({fileChanges.length})
            </span>
          )}
        </button>
      </div>

      {/* Content area */}
      {activeTab === "activity" ? (
        <AgentTimeline events={events} isThinking={isThinking} />
      ) : (
        <AgentFiles fileChanges={fileChanges} />
      )}
    </aside>
  );
}
