import React, { useCallback, useState, useMemo } from "react";
import { open } from "@tauri-apps/plugin-dialog";
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
      style={{
        width: 240,
        background: "var(--color-surface)",
        borderRight: "1px solid var(--color-border-muted)",
      }}
      className="flex flex-col select-none shrink-0"
    >
      {/* ── Header ── */}
      <div
        style={{
          height: 48,
          padding: "0 var(--sp-2) 0 var(--sp-3)",
          borderBottom: "1px solid var(--color-border-muted)",
        }}
        className="flex items-center gap-1 shrink-0"
      >
        {/* Open Folder */}
        <button
          onClick={handleOpenFolder}
          title="Open Folder"
          className="flex items-center gap-1.5 hover:bg-[var(--color-surface-hover)] transition-colors"
          style={{
            height: 28,
            padding: "0 8px",
            borderRadius: "var(--radius-sm)",
            color: "var(--color-text)",
            fontSize: 13,
            fontWeight: 600,
          }}
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round">
            <path d="M2 4.5C2 3.67 2.67 3 3.5 3H6l1.5 1.5H12.5C13.33 4.5 14 5.17 14 6V11.5C14 12.33 13.33 13 12.5 13H3.5C2.67 13 2 12.33 2 11.5V4.5Z" />
          </svg>
          Wit
        </button>

        <div className="flex-1" />

        {/* Global Search / Command Palette */}
        <button
          onClick={onOpenPalette}
          title="Command Palette (Ctrl+Shift+P)"
          style={{ width: 28, height: 28, borderRadius: "var(--radius-md)", color: "var(--color-text-muted)" }}
          className="flex items-center justify-center hover:text-[var(--color-text)] hover:bg-[var(--color-surface-hover)] transition-colors"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
            <circle cx="6" cy="6" r="4" />
            <line x1="9" y1="9" x2="12.5" y2="12.5" />
          </svg>
        </button>

        {/* Settings */}
        <button
          onClick={onOpenSettings}
          title="Settings (Ctrl+,)"
          style={{ width: 28, height: 28, borderRadius: "var(--radius-md)", color: "var(--color-text-muted)" }}
          className="flex items-center justify-center hover:text-[var(--color-text)] hover:bg-[var(--color-surface-hover)] transition-colors"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
            <path d="M7 9a2 2 0 100-4 2 2 0 000 4zm5.5-1.6l-.98-.57a4.4 4.4 0 000-1.66l.98-.57a.44.44 0 00.16-.6l-.88-1.52a.44.44 0 00-.6-.16l-.98.57a4.4 4.4 0 00-1.43-.83V1.5a.44.44 0 00-.44-.44H6.57a.44.44 0 00-.44.44v.56a4.4 4.4 0 00-1.43.83l-.98-.57a.44.44 0 00-.6.16L2.24 4a.44.44 0 00.16.6l.98.57a4.4 4.4 0 000 1.66l-.98.57a.44.44 0 00-.16.6l.88 1.52a.44.44 0 00.6.16l.98-.57a4.4 4.4 0 001.43.83v.56c0 .24.2.44.44.44h1.76c.24 0 .44-.2.44-.44v-.56a4.4 4.4 0 001.43-.83l.98.57a.44.44 0 00.6-.16l.88-1.52a.44.44 0 00-.16-.6z" />
          </svg>
        </button>
      </div>

      {/* ── Search bar ── */}
      <div style={{ padding: "var(--sp-2) var(--sp-3)" }} className="shrink-0">
        <div
          style={{
            height: 30,
            background: "var(--color-bg)",
            border: "1px solid var(--color-border)",
            borderRadius: "var(--radius-sm)",
            padding: "0 8px",
          }}
          className="flex items-center gap-1.5"
        >
          <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="var(--color-text-muted)" strokeWidth="1.3">
            <circle cx="5.5" cy="5.5" r="3.5" />
            <line x1="8.2" y1="8.2" x2="11.5" y2="11.5" />
          </svg>
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Filter sessions..."
            spellCheck={false}
            style={{
              background: "transparent",
              border: "none",
              outline: "none",
              color: "var(--color-text)",
              fontSize: 12,
            }}
            className="flex-1 min-w-0 placeholder:text-[var(--color-text-muted)]"
          />
        </div>
      </div>

      {/* ── Session list ── */}
      <div role="listbox" className="flex-1 overflow-y-auto" style={{ padding: "var(--sp-1) 0" }}>
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
      <button
        onClick={handleNewSession}
        style={{
          height: 40,
          borderTop: "1px solid var(--color-border-muted)",
          color: "var(--color-text-secondary)",
          fontSize: 13,
        }}
        className="flex items-center justify-center gap-2 shrink-0 hover:text-[var(--color-text)] hover:bg-[var(--color-surface-hover)] transition-colors"
      >
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
          <line x1="7" y1="2" x2="7" y2="12" />
          <line x1="2" y1="7" x2="12" y2="7" />
        </svg>
        New Session
      </button>
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
        height: 36,
        margin: "0 var(--sp-2)",
        padding: "var(--sp-2) var(--sp-3)",
        borderRadius: "var(--radius-sm)",
        borderLeft: isActive ? "2px solid var(--color-primary)" : "2px solid transparent",
        background: isActive ? "var(--color-surface-active)" : "transparent",
        color: isActive ? "var(--color-text)" : "var(--color-text-secondary)",
        cursor: "pointer",
        transition: "var(--transition-fast)",
      }}
      className="group flex items-center gap-2"
      onMouseEnter={(e) => { if (!isActive) e.currentTarget.style.background = "var(--color-surface-hover)"; }}
      onMouseLeave={(e) => { if (!isActive) e.currentTarget.style.background = "transparent"; }}
    >
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none"
        stroke={isActive ? "var(--color-primary)" : "currentColor"} strokeWidth="1.5" className="shrink-0">
        <polyline points="4,4 8,8 4,12" /><line x1="9" y1="12" x2="13" y2="12" />
      </svg>
      <div className="flex-1 min-w-0">
        <div style={{ fontSize: 13, lineHeight: "18px", fontWeight: isActive ? 500 : 400 }} className="truncate">{title}</div>
        {cwd && <div style={{ fontSize: 11, lineHeight: "14px", color: "var(--color-text-muted)" }} className="truncate">{cwd}</div>}
      </div>
      <button onClick={handleClose}
        style={{ width: 20, height: 20, borderRadius: "var(--radius-sm)", color: "var(--color-text-muted)" }}
        className="shrink-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity hover:text-[var(--color-error)] hover:bg-[var(--color-surface-active)]">
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.5">
          <line x1="2" y1="2" x2="8" y2="8" /><line x1="8" y1="2" x2="2" y2="8" />
        </svg>
      </button>
    </div>
  );
});

function cwdBasename(cwd: string): string {
  if (!cwd) return "";
  return cwd.replace(/\\/g, "/").split("/").filter(Boolean).pop() || "";
}
