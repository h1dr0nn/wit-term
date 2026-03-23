import { useState, useCallback, useRef, useMemo } from "react";
import { useSessionStore } from "../stores/sessionStore";

interface Command {
  id: string;
  label: string;
  shortcut?: string;
  action: () => void;
  category: string;
}

interface CommandPaletteProps {
  visible: boolean;
  onClose: () => void;
  onOpenSettings: () => void;
  onToggleSidebar: () => void;
  onToggleContextSidebar: () => void;
}

export function CommandPalette({
  visible,
  onClose,
  onOpenSettings,
  onToggleSidebar,
  onToggleContextSidebar,
}: CommandPaletteProps) {
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const createNewSession = useSessionStore((s) => s.createNewSession);
  const closeSession = useSessionStore((s) => s.closeSession);
  const activeSessionId = useSessionStore((s) => s.activeSessionId);
  const switchToNext = useSessionStore((s) => s.switchToNext);
  const switchToPrevious = useSessionStore((s) => s.switchToPrevious);

  const commands: Command[] = useMemo(
    () => [
      {
        id: "new-session",
        label: "New Terminal Session",
        shortcut: "Ctrl+T",
        action: () => createNewSession(),
        category: "Session",
      },
      {
        id: "close-session",
        label: "Close Current Session",
        shortcut: "Ctrl+W",
        action: () => {
          if (activeSessionId) closeSession(activeSessionId);
        },
        category: "Session",
      },
      {
        id: "next-session",
        label: "Switch to Next Session",
        shortcut: "Ctrl+Tab",
        action: switchToNext,
        category: "Session",
      },
      {
        id: "prev-session",
        label: "Switch to Previous Session",
        shortcut: "Ctrl+Shift+Tab",
        action: switchToPrevious,
        category: "Session",
      },
      {
        id: "toggle-sidebar",
        label: "Toggle Session Sidebar",
        shortcut: "Ctrl+B",
        action: onToggleSidebar,
        category: "View",
      },
      {
        id: "toggle-context",
        label: "Toggle Context Sidebar",
        shortcut: "Ctrl+Shift+B",
        action: onToggleContextSidebar,
        category: "View",
      },
      {
        id: "open-settings",
        label: "Open Settings",
        shortcut: "Ctrl+,",
        action: onOpenSettings,
        category: "Preferences",
      },
    ],
    [
      createNewSession,
      closeSession,
      activeSessionId,
      switchToNext,
      switchToPrevious,
      onOpenSettings,
      onToggleSidebar,
      onToggleContextSidebar,
    ],
  );

  const filtered = useMemo(() => {
    if (!query.trim()) return commands;
    const q = query.toLowerCase();
    return commands.filter(
      (cmd) =>
        cmd.label.toLowerCase().includes(q) ||
        cmd.category.toLowerCase().includes(q),
    );
  }, [commands, query]);

  // Focus input when palette becomes visible via callback ref
  const setInputRef = useCallback((el: HTMLInputElement | null) => {
    (inputRef as React.MutableRefObject<HTMLInputElement | null>).current = el;
    if (el && visible) {
      el.focus();
    }
  }, [visible]);

  const executeCommand = useCallback(
    (cmd: Command) => {
      onClose();
      // Delay action to let modal close first
      setTimeout(() => cmd.action(), 50);
    },
    [onClose],
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        onClose();
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((i) => Math.min(i + 1, filtered.length - 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((i) => Math.max(i - 1, 0));
      } else if (e.key === "Enter") {
        e.preventDefault();
        if (filtered[selectedIndex]) {
          executeCommand(filtered[selectedIndex]);
        }
      }
    },
    [onClose, filtered, selectedIndex, executeCommand],
  );

  if (!visible) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-black/30"
      onClick={onClose}
    >
      <div
        className="w-[480px] bg-[var(--ui-bg)] border border-[var(--ui-border)] rounded-lg shadow-2xl overflow-hidden"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={handleKeyDown}
      >
        <input
          ref={setInputRef}
          type="text"
          value={query}
          onChange={(e) => { setQuery(e.target.value); setSelectedIndex(0); }}
          className="w-full bg-transparent border-b border-[var(--ui-border)] px-4 py-3 text-sm text-[var(--ui-fg)] outline-none placeholder:text-[var(--ui-fg-dim)]"
          placeholder="Type a command..."
        />
        <div className="max-h-72 overflow-y-auto py-1">
          {filtered.map((cmd, idx) => (
            <div
              key={cmd.id}
              onClick={() => executeCommand(cmd)}
              className={`flex items-center justify-between px-4 py-2 cursor-pointer ${
                idx === selectedIndex
                  ? "bg-[var(--ui-bg-tertiary)]"
                  : "hover:bg-[var(--ui-bg-secondary)]"
              }`}
            >
              <div className="flex items-center gap-3">
                <span className="text-xs text-[var(--ui-fg-dim)] w-20">
                  {cmd.category}
                </span>
                <span className="text-sm text-[var(--ui-fg)]">{cmd.label}</span>
              </div>
              {cmd.shortcut && (
                <span className="text-xs text-[var(--ui-fg-dim)] bg-[var(--ui-bg-secondary)] px-1.5 py-0.5 rounded">
                  {cmd.shortcut}
                </span>
              )}
            </div>
          ))}
          {filtered.length === 0 && (
            <div className="px-4 py-6 text-center text-[var(--ui-fg-dim)] text-sm">
              No matching commands
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
