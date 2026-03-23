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
    <aside className="w-52 bg-[#181825] border-r border-[#313244] flex flex-col select-none">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-[#313244]">
        <span className="text-xs font-semibold text-[#a6adc8] uppercase tracking-wider">
          Sessions
        </span>
        <button
          onClick={handleNewSession}
          className="text-[#6c7086] hover:text-[#cdd6f4] text-lg leading-none px-1"
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
          <div className="px-3 py-4 text-center text-[#585b70] text-xs">
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
          ? "bg-[#313244] text-[#cdd6f4]"
          : "text-[#a6adc8] hover:bg-[#1e1e2e]"
      }`}
    >
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1">
          <span className="text-xs text-[#585b70]">{index + 1}</span>
          <span className="text-sm truncate">{title}</span>
        </div>
        {cwd && (
          <div className="text-xs text-[#585b70] truncate">{cwd}</div>
        )}
      </div>
      <button
        onClick={handleClose}
        className="shrink-0 w-4 h-4 flex items-center justify-center rounded text-[#585b70] hover:text-[#f38ba8] hover:bg-[#45475a] opacity-0 group-hover:opacity-100 transition-opacity text-xs"
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
  // Try to show last 2 segments
  const parts = normalized.split("/").filter(Boolean);
  if (parts.length <= 2) return normalized;
  return `${home}/${parts.slice(-2).join("/")}`;
}
