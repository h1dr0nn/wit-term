import type { AgentFileChange } from "../../stores/agentStore";

function formatRelativeTime(timestamp: number): string {
  const now = Date.now();
  const diff = Math.max(0, Math.floor((now - timestamp) / 1000));
  if (diff < 5) return "just now";
  if (diff < 60) return `${diff}s ago`;
  const mins = Math.floor(diff / 60);
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  return `${hours}h ago`;
}

function actionIcon(action: AgentFileChange["action"]): { symbol: string; colorClass: string } {
  switch (action) {
    case "created":
      return { symbol: "+", colorClass: "text-green-400" };
    case "modified":
      return { symbol: "~", colorClass: "text-yellow-400" };
    case "deleted":
      return { symbol: "-", colorClass: "text-red-400" };
  }
}

function fileName(path: string): string {
  const parts = path.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || path;
}

interface AgentFilesProps {
  fileChanges: AgentFileChange[];
}

export function AgentFiles({ fileChanges }: AgentFilesProps) {
  if (fileChanges.length === 0) {
    return (
      <div className="py-20 px-4 text-center text-xs text-[var(--color-text-muted)]">
        No file changes
      </div>
    );
  }

  const created = fileChanges.filter((f) => f.action === "created").length;
  const modified = fileChanges.filter((f) => f.action === "modified").length;
  const deleted = fileChanges.filter((f) => f.action === "deleted").length;

  return (
    <div className="flex-1 overflow-y-auto custom-scrollbar">
      {/* Summary bar */}
      <div className="px-3 py-2 border-b border-[var(--color-border-muted)] text-xs text-[var(--color-text-muted)] flex items-center gap-2">
        <span>{fileChanges.length} files changed</span>
        {created > 0 && <span className="text-green-400">+{created}</span>}
        {modified > 0 && <span className="text-yellow-400">~{modified}</span>}
        {deleted > 0 && <span className="text-red-400">-{deleted}</span>}
      </div>

      {/* File list */}
      <div className="px-3 py-1">
        {fileChanges.map((change, i) => {
          const icon = actionIcon(change.action);
          return (
            <div key={`${change.path}-${i}`} className="flex items-center gap-2 py-1.5">
              <span
                className={`flex-shrink-0 w-4 text-center font-mono text-xs font-bold ${icon.colorClass}`}
              >
                {icon.symbol}
              </span>
              <span
                className="flex-1 text-xs text-[var(--color-text-secondary)] truncate font-mono"
                title={change.path}
              >
                {fileName(change.path)}
              </span>
              <span className="flex-shrink-0 text-[10px] text-[var(--color-text-muted)]">
                {formatRelativeTime(change.timestamp)}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
