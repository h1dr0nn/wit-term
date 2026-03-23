import React, { useCallback } from "react";
import { useSessionStore, type SessionInfo } from "../../stores/sessionStore";

export function TabBar() {
  const sessions = useSessionStore((s) => s.sessions);
  const activeSessionId = useSessionStore((s) => s.activeSessionId);
  const setActiveSession = useSessionStore((s) => s.setActiveSession);
  const createNewSession = useSessionStore((s) => s.createNewSession);
  const closeSession = useSessionStore((s) => s.closeSession);

  const handleNewTab = useCallback(() => {
    createNewSession();
  }, [createNewSession]);

  return (
    <div className="flex items-center bg-[#181825] border-b border-[#313244] h-9 select-none">
      <div className="flex-1 flex items-center overflow-x-auto scrollbar-none">
        {sessions.map((session, idx) => (
          <Tab
            key={session.id}
            session={session}
            index={idx}
            isActive={session.id === activeSessionId}
            onSelect={setActiveSession}
            onClose={closeSession}
          />
        ))}
      </div>
      <button
        onClick={handleNewTab}
        className="px-3 h-full text-[#6c7086] hover:text-[#cdd6f4] hover:bg-[#313244] transition-colors text-lg leading-none"
        title="New tab (Ctrl+T)"
      >
        +
      </button>
    </div>
  );
}

interface TabProps {
  session: SessionInfo;
  index: number;
  isActive: boolean;
  onSelect: (id: string) => void;
  onClose: (id: string) => void;
}

const Tab = React.memo(function Tab({
  session,
  index,
  isActive,
  onSelect,
  onClose,
}: TabProps) {
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

  // Show tab title: session title, or CWD basename, or index
  const displayTitle = session.title || cwdBasename(session.cwd) || `Terminal ${index + 1}`;

  return (
    <div
      onClick={handleClick}
      className={`group flex items-center gap-1 px-3 h-full cursor-pointer border-r border-[#313244] min-w-0 max-w-48 ${
        isActive
          ? "bg-[#1e1e2e] text-[#cdd6f4] border-b-2 border-b-[#89b4fa]"
          : "text-[#6c7086] hover:text-[#a6adc8] hover:bg-[#1e1e2e]/50"
      }`}
      title={session.cwd || session.title}
    >
      <span className="text-xs text-[#585b70] shrink-0">{index + 1}</span>
      <span className="truncate text-sm">{displayTitle}</span>
      <button
        onClick={handleClose}
        className="ml-auto shrink-0 w-4 h-4 flex items-center justify-center rounded text-[#585b70] hover:text-[#f38ba8] hover:bg-[#45475a] opacity-0 group-hover:opacity-100 transition-opacity text-xs"
        title="Close tab"
      >
        x
      </button>
    </div>
  );
});

function cwdBasename(cwd: string): string {
  if (!cwd) return "";
  const parts = cwd.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || parts[parts.length - 2] || "";
}
