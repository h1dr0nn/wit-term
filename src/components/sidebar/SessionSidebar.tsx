import React, { useCallback, useState, useMemo } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { 
  FolderOpen, 
  Search, 
  Settings, 
  Plus, 
  Terminal, 
  X 
} from "lucide-react";

import { useSessionStore, type SessionInfo } from "../../stores/sessionStore";

interface SessionSidebarProps {
  onOpenSettings: () => void;
  onOpenPalette: () => void;
}

export function SessionSidebar({ onOpenSettings, onOpenPalette }: SessionSidebarProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const sessions = useSessionStore((s) => s.sessions);
  const activeSessionId = useSessionStore((s) => s.activeSessionId);
  const setActiveSession = useSessionStore((s) => s.setActiveSession);
  const createNewSession = useSessionStore((s) => s.createNewSession);
  const closeSession = useSessionStore((s) => s.closeSession);

  const filteredSessions = useMemo(() => {
    if (!searchQuery.trim()) return sessions;
    const q = searchQuery.toLowerCase();
    return sessions.filter(
      (s) => s.title.toLowerCase().includes(q) || s.cwd.toLowerCase().includes(q),
    );
  }, [sessions, searchQuery]);

  const handleNewSession = useCallback(() => {
    createNewSession();
  }, [createNewSession]);

  const handleOpenFolder = useCallback(async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Open Project Folder",
    });
    if (selected) {
      createNewSession(selected as string);
    }
  }, [createNewSession]);

  return (
    <aside
      role="complementary"
      aria-label="Sessions"
      className="flex flex-col select-none shrink-0 border-r border-[var(--color-border-muted)] bg-[var(--color-surface)]"
      style={{ width: 240 }}
    >
      {/* ── Header ── */}
      <div
        className="flex items-center gap-1 shrink-0 h-12 px-3 border-b border-[var(--color-border-muted)]"
      >
        {/* Open Folder */}
        <button
          onClick={handleOpenFolder}
          title="Open Folder"
          className="flex items-center gap-1.5 h-8 px-2.5 rounded-lg text-sm font-bold text-[var(--color-text)] hover:bg-[var(--color-surface-hover)] transition-all active:scale-95"
        >
          <FolderOpen size={16} strokeWidth={2.5} className="text-[var(--color-primary)]" />
          Wit
        </button>

        <div className="flex-1" />

        {/* Global Search / Command Palette */}
        <button
          onClick={onOpenPalette}
          title="Command Palette (Ctrl+Shift+P)"
          className="w-8 h-8 flex items-center justify-center rounded-lg text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-surface-hover)] transition-all"
        >
          <Search size={14} strokeWidth={2} />
        </button>

        {/* Settings */}
        <button
          onClick={onOpenSettings}
          title="Settings (Ctrl+,)"
          className="w-8 h-8 flex items-center justify-center rounded-lg text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-surface-hover)] transition-all"
        >
          <Settings size={14} strokeWidth={2} />
        </button>
      </div>

      {/* ── Search bar ── */}
      <div className="shrink-0 p-3">
        <div
          className="flex items-center gap-1.5 h-8 px-3 bg-[var(--color-bg)] border border-[var(--color-border)] rounded-lg focus-within:border-[var(--color-primary-muted)] focus-within:ring-1 focus-within:ring-[var(--color-primary-muted)] transition-all"
        >
          <Search size={13} strokeWidth={2} className="text-[var(--color-text-muted)]" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Filter sessions..."
            spellCheck={false}
            className="flex-1 min-w-0 bg-transparent border-none outline-none text-xs text-[var(--color-text)] placeholder:text-[var(--color-text-muted)]"
          />
        </div>
      </div>

      {/* ── Session list ── */}
      <div role="listbox" className="flex-1 overflow-y-auto py-1 custom-scrollbar">
        {filteredSessions.map((session, idx) => (
          <SessionRow
            key={session.id}
            session={session}
            index={idx}
            isActive={session.id === activeSessionId}
            onSelect={setActiveSession}
            onClose={closeSession}
          />
        ))}
      </div>

      {/* ── Footer: New Session ── */}
      <div className="p-3 mt-auto border-t border-[var(--color-border-muted)]">
        <button
          onClick={handleNewSession}
          className="w-full flex items-center justify-center gap-2 h-9 rounded-lg text-sm font-medium text-[var(--color-text-secondary)] hover:text-[var(--color-text)] hover:bg-[var(--color-surface-hover)] transition-all active:scale-[0.98]"
        >
          <Plus size={14} strokeWidth={2.5} />
          New Session
        </button>
      </div>
    </aside>

  );
}

/* ── Session Row ── */

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
  const handleClick = useCallback(() => onSelect(session.id), [onSelect, session.id]);
  const handleClose = useCallback(
    (e: React.MouseEvent) => { e.stopPropagation(); onClose(session.id); },
    [onClose, session.id],
  );

  const title = session.title || `Session ${index + 1}`;
  const cwd = cwdBasename(session.cwd);

  return (
    <div
      role="option"
      aria-selected={isActive}
      onClick={handleClick}
      style={{
        height: 48,
        margin: "2px var(--sp-2)",
        padding: "0 var(--sp-3)",
        borderRadius: "var(--radius-md)",
        background: isActive ? "var(--color-surface-active)" : "transparent",
        color: isActive ? "var(--color-text)" : "var(--color-text-secondary)",
        cursor: "pointer",
        transition: "var(--transition-fast)",
      }}
      className="group flex items-center gap-3"
      onMouseEnter={(e) => { if (!isActive) e.currentTarget.style.background = "var(--color-surface-hover)"; }}
      onMouseLeave={(e) => { if (!isActive) e.currentTarget.style.background = "transparent"; }}
    >
      <Terminal 
        size={18} 
        strokeWidth={isActive ? 2.5 : 2} 
        color={isActive ? "var(--color-primary)" : "var(--color-text-muted)"} 
        className="shrink-0" 
      />
      <div className="flex-1 min-w-0 flex flex-col justify-center gap-0.5">
        <div style={{ fontSize: 13.5, fontWeight: 700, color: "var(--color-text)" }} className="truncate">
          {cwd || "New Session"}
        </div>
        <div style={{ fontSize: 10.5, color: "var(--color-text-muted)", opacity: 0.8 }} className="truncate">
          {title}
        </div>
      </div>
      <button onClick={handleClose}
        style={{ width: 20, height: 20, borderRadius: "var(--radius-sm)", color: "var(--color-text-muted)" }}
        className="shrink-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity hover:text-[var(--color-error)] hover:bg-[var(--color-surface-active)]">
        <X size={10} strokeWidth={2.5} />
      </button>
    </div>

  );
});

function cwdBasename(cwd: string): string {
  if (!cwd) return "";
  return cwd.replace(/\\/g, "/").split("/").filter(Boolean).pop() || "";
}
