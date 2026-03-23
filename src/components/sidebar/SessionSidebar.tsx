import React, { useCallback } from "react";
import { useSessionStore, type SessionInfo } from "../../stores/sessionStore";

export function SessionSidebar() {
  const sessions = useSessionStore((s) => s.sessions);
  const activeSessionId = useSessionStore((s) => s.activeSessionId);
  const setActiveSession = useSessionStore((s) => s.setActiveSession);
  const createNewSession = useSessionStore((s) => s.createNewSession);
  const closeSession = useSessionStore((s) => s.closeSession);

  const handleNewSession = useCallback(() => {
    createNewSession();
  }, [createNewSession]);

  return (
    <aside className="w-52 bg-[var(--ui-bg-secondary)] border-r border-[var(--ui-border)] flex flex-col select-none">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-[var(--ui-border)]">
        <span className="text-xs font-semibold text-[var(--ui-fg-muted)] uppercase tracking-wider">
          Sessions
        </span>
        <button
          onClick={handleNewSession}
          className="text-[var(--ui-fg-dim)] hover:text-[var(--ui-fg)] text-lg leading-none px-1"
          title="New session"
        >
          +
        </button>
      </div>

      {/* Session list */}
      <div className="flex-1 overflow-y-auto py-1">
        {sessions.map((session, idx) => (
          <SessionRow
            key={session.id}
            session={session}
            index={idx}
            isActive={session.id === activeSessionId}
            onSelect={setActiveSession}
            onClose={closeSession}
          />
        ))}
        {sessions.length === 0 && (
          <div className="px-3 py-4 text-center text-[var(--ui-fg-dim)] text-xs">
            No sessions
          </div>
        )}
      </div>
    </aside>
  );
}

interface SessionRowProps {
  session: SessionInfo;
  index: number;
  isActive: boolean;
  onSelect: (id: string) => void;
  onClose: (id: string) => void;
}

const SessionRow = React.memo(function SessionRow({
  session,
  index,
  isActive,
  onSelect,
  onClose,
}: SessionRowProps) {
  const handleClick = useCallback(() => {
    onSelect(session.id);
  }, [onSelect, session.id]);

  const handleClose = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      onClose(session.id);
    },
    [onClose, session.id],
  );

  const title = session.title || `Terminal ${index + 1}`;
  const cwd = cwdShort(session.cwd);

  return (
    <div
      onClick={handleClick}
      className={`group flex items-center gap-2 px-3 py-1.5 cursor-pointer mx-1 rounded ${
        isActive
          ? "bg-[var(--ui-bg-tertiary)] text-[var(--ui-fg)]"
          : "text-[var(--ui-fg-muted)] hover:bg-[var(--ui-bg)]"
      }`}
    >
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1">
          <span className="text-xs text-[var(--ui-fg-dim)]">{index + 1}</span>
          <span className="text-sm truncate">{title}</span>
        </div>
        {cwd && (
          <div className="text-xs text-[var(--ui-fg-dim)] truncate">{cwd}</div>
        )}
      </div>
      <button
        onClick={handleClose}
        className="shrink-0 w-4 h-4 flex items-center justify-center rounded text-[var(--ui-fg-dim)] hover:text-[var(--term-red)] hover:bg-[var(--ui-border)] opacity-0 group-hover:opacity-100 transition-opacity text-xs"
      >
        x
      </button>
    </div>
  );
});

function cwdShort(cwd: string): string {
  if (!cwd) return "";
  const normalized = cwd.replace(/\\/g, "/");
  const home = "~";
  const parts = normalized.split("/").filter(Boolean);
  if (parts.length <= 2) return normalized;
  return `${home}/${parts.slice(-2).join("/")}`;
}
