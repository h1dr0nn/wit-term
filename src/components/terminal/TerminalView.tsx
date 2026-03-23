import { useEffect, useRef, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { BlocksView } from "./BlocksView";
import { InputBar } from "./InputBar";
import { CompletionPopup } from "./CompletionPopup";
import { SearchOverlay, type SearchOptions } from "./SearchOverlay";
import { useTerminalStore, type GridSnapshot } from "../../stores/terminalStore";
import { useCompletionStore, type CompletionItem } from "../../stores/completionStore";
import { useSessionStore } from "../../stores/sessionStore";
import { useInputBuffer } from "../../hooks/useInputBuffer";
import { encodeKey } from "../../utils/keyEncoder";

interface GridUpdatePayload {
  session_id: string;
  snapshot: GridSnapshot;
}

interface SessionExitedPayload {
  session_id: string;
  exit_code: number;
}

interface CwdChangedPayload {
  session_id: string;
  cwd: string;
}

interface TitleChangedPayload {
  session_id: string;
  title: string;
}

export function TerminalView() {
  const sessionId = useSessionStore((s) => s.activeSessionId);
  const updateSessionTitle = useSessionStore((s) => s.updateSessionTitle);
  const updateSessionCwd = useSessionStore((s) => s.updateSessionCwd);

  const [exitedSessions, setExitedSessions] = useState<Set<string>>(
    () => new Set(),
  );
  const [searchVisible, setSearchVisible] = useState(false);
  const [searchMatches, setSearchMatches] = useState(0);
  const [searchCurrent, setSearchCurrent] = useState(0);
  const cwdRef = useRef<string>("");
  const [currentCwd, setCurrentCwd] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);
  const updateGrid = useTerminalStore((s) => s.updateGrid);
  const grids = useTerminalStore((s) => s.grids);

  const completionVisible = useCompletionStore((s) => s.visible);
  const completionShow = useCompletionStore((s) => s.show);
  const completionHide = useCompletionStore((s) => s.hide);
  const completionSelectNext = useCompletionStore((s) => s.selectNext);
  const completionSelectPrevious = useCompletionStore((s) => s.selectPrevious);
  const completionGetSelected = useCompletionStore((s) => s.getSelected);

  const { append, reset, getBuffer, getCursor, insertCompletion } =
    useInputBuffer();

  const snapshot = sessionId ? grids.get(sessionId) : undefined;
  const exited = sessionId ? exitedSessions.has(sessionId) : false;

  // Focus container when active session changes
  useEffect(() => {
    if (sessionId) {
      containerRef.current?.focus();
      completionHide();
      reset();
    }
  }, [sessionId, completionHide, reset]);

  // Listen for events (all sessions)
  useEffect(() => {
    const unlisten1 = listen<GridUpdatePayload>("grid_update", (event) => {
      updateGrid(event.payload.session_id, event.payload.snapshot);
    });

    const unlisten2 = listen<SessionExitedPayload>(
      "session_exited",
      (event) => {
        setExitedSessions((prev) => new Set(prev).add(event.payload.session_id));
      },
    );

    const unlisten3 = listen<CwdChangedPayload>("cwd_changed", (event) => {
      updateSessionCwd(event.payload.session_id, event.payload.cwd);
      // Update local ref if this is the active session
      if (event.payload.session_id === sessionId) {
        cwdRef.current = event.payload.cwd;
        setCurrentCwd(event.payload.cwd);
        reset();
      }
    });

    const unlisten4 = listen<TitleChangedPayload>(
      "title_changed",
      (event) => {
        updateSessionTitle(event.payload.session_id, event.payload.title);
      },
    );

    return () => {
      unlisten1.then((fn) => fn());
      unlisten2.then((fn) => fn());
      unlisten3.then((fn) => fn());
      unlisten4.then((fn) => fn());
    };
  }, [updateGrid, updateSessionCwd, updateSessionTitle, sessionId, reset]);

  // Handle resize
  useEffect(() => {
    if (!sessionId || !containerRef.current) return;

    const observer = new ResizeObserver(() => {
      if (!containerRef.current || !sessionId) return;

      const rect = containerRef.current.getBoundingClientRect();
      const charWidth = 8.4;
      const charHeight = 14 * 1.2;

      const cols = Math.max(1, Math.floor(rect.width / charWidth));
      const rows = Math.max(1, Math.floor(rect.height / charHeight));

      invoke("resize_session", { sessionId, cols, rows }).catch(() => {});
    });

    observer.observe(containerRef.current);
    return () => observer.disconnect();
  }, [sessionId]);

  // Search handlers
  const handleSearch = useCallback(
    (query: string, _options: SearchOptions) => {
      if (!snapshot || !query) {
        setSearchMatches(0);
        setSearchCurrent(0);
        return;
      }
      // Simple text search across grid rows
      let count = 0;
      for (const row of snapshot.rows) {
        const text = row.map((c) => c.content || " ").join("");
        let pos = 0;
        const lowerText = text.toLowerCase();
        const lowerQuery = query.toLowerCase();
        while (pos < lowerText.length) {
          const idx = lowerText.indexOf(lowerQuery, pos);
          if (idx === -1) break;
          count++;
          pos = idx + 1;
        }
      }
      setSearchMatches(count);
      setSearchCurrent(count > 0 ? 0 : -1);
    },
    [snapshot],
  );

  const handleSearchNext = useCallback(() => {
    if (searchMatches > 0) {
      setSearchCurrent((prev) => (prev + 1) % searchMatches);
    }
  }, [searchMatches]);

  const handleSearchPrevious = useCallback(() => {
    if (searchMatches > 0) {
      setSearchCurrent(
        (prev) => (prev - 1 + searchMatches) % searchMatches,
      );
    }
  }, [searchMatches]);

  const handleSearchClose = useCallback(() => {
    setSearchVisible(false);
    setSearchMatches(0);
    setSearchCurrent(0);
    containerRef.current?.focus();
  }, []);

  // Request completions from backend
  const requestCompletions = useCallback(async () => {
    const input = getBuffer();
    const cursorPos = getCursor();
    const cwd = cwdRef.current;

    if (!input.trim()) return;

    try {
      const items = await invoke<CompletionItem[]>("request_completions", {
        input,
        cursorPos,
        cwd,
      });

      if (items.length === 1) {
        const data = insertCompletion(items[0].text);
        if (sessionId && data) {
          await invoke("send_input", { sessionId, data });
        }
      } else if (items.length > 1) {
        completionShow(items);
      }
    } catch {
      // Completion request failed
    }
  }, [getBuffer, getCursor, insertCompletion, sessionId, completionShow]);

  // Handle keyboard input
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (!sessionId || exited) return;

      // Global shortcuts (work regardless of completion state)
      const sessions = useSessionStore.getState().sessions;

      // Ctrl+T = new tab
      if (e.ctrlKey && !e.shiftKey && (e.key === "t" || e.key === "T")) {
        e.preventDefault();
        useSessionStore.getState().createNewSession();
        return;
      }

      // Ctrl+W = close tab
      if (e.ctrlKey && !e.shiftKey && (e.key === "w" || e.key === "W")) {
        e.preventDefault();
        useSessionStore.getState().closeSession(sessionId);
        return;
      }

      // Ctrl+Tab / Ctrl+Shift+Tab = switch tabs
      if (e.ctrlKey && e.key === "Tab") {
        e.preventDefault();
        if (e.shiftKey) {
          useSessionStore.getState().switchToPrevious();
        } else {
          useSessionStore.getState().switchToNext();
        }
        return;
      }

      // Ctrl+1-9 = switch to tab by index
      if (e.ctrlKey && !e.shiftKey && e.key >= "1" && e.key <= "9") {
        const idx = parseInt(e.key) - 1;
        if (idx < sessions.length) {
          e.preventDefault();
          useSessionStore.getState().switchToIndex(idx);
          return;
        }
      }

      // Completion popup keyboard handling
      if (completionVisible) {
        if (e.key === "ArrowDown") {
          e.preventDefault();
          completionSelectNext();
          return;
        }
        if (e.key === "ArrowUp") {
          e.preventDefault();
          completionSelectPrevious();
          return;
        }
        if (e.key === "Enter" || e.key === "Tab") {
          e.preventDefault();
          const selected = completionGetSelected();
          if (selected) {
            const data = insertCompletion(selected.text);
            if (data) {
              invoke("send_input", { sessionId, data }).catch(() => {});
            }
          }
          completionHide();
          return;
        }
        if (e.key === "Escape") {
          e.preventDefault();
          completionHide();
          return;
        }
      }

      // Tab key (when popup not visible) -> trigger completions
      if (
        e.key === "Tab" &&
        !e.shiftKey &&
        !e.ctrlKey &&
        !e.altKey &&
        !e.metaKey
      ) {
        e.preventDefault();
        requestCompletions();
        return;
      }

      // Ctrl+Shift+F = search
      if (e.ctrlKey && e.shiftKey && (e.key === "F" || e.key === "f")) {
        e.preventDefault();
        setSearchVisible((v) => !v);
        return;
      }

      // Clipboard: Ctrl+Shift+V = paste
      if (e.ctrlKey && e.shiftKey && (e.key === "V" || e.key === "v")) {
        e.preventDefault();
        navigator.clipboard
          .readText()
          .then((text) => {
            if (text) {
              append(text);
              invoke("send_input", { sessionId, data: text }).catch(() => {});
            }
          })
          .catch(() => {});
        return;
      }

      // Clipboard: Ctrl+Shift+C = copy selection
      if (e.ctrlKey && e.shiftKey && (e.key === "C" || e.key === "c")) {
        e.preventDefault();
        const selection = window.getSelection()?.toString();
        if (selection) {
          navigator.clipboard.writeText(selection).catch(() => {});
        }
        return;
      }

      e.preventDefault();

      const data = encodeKey(e);
      if (data) {
        append(data);
        invoke("send_input", { sessionId, data }).catch((err) => {
          console.error("Failed to send input:", err);
        });
      }
    },
    [
      sessionId,
      exited,
      completionVisible,
      completionSelectNext,
      completionSelectPrevious,
      completionGetSelected,
      completionHide,
      requestCompletions,
      insertCompletion,
      append,
    ],
  );

  const handleClick = useCallback(() => {
    containerRef.current?.focus();
  }, []);

  const handleInputSubmit = useCallback(
    (input: string) => {
      if (!sessionId) return;
      invoke("send_input", { sessionId, data: input + "\r" }).catch(() => {});
      append(input + "\r");
    },
    [sessionId, append],
  );

  const handleInputTab = useCallback(
    (input: string, cursorPos: number) => {
      const cwd = cwdRef.current;
      if (!input.trim()) return;
      invoke<CompletionItem[]>("request_completions", { input, cursorPos, cwd })
        .then((items) => {
          if (items.length > 0) completionShow(items);
        })
        .catch(() => {});
    },
    [completionShow],
  );

  const handleRerun = useCallback(
    (command: string) => {
      if (!sessionId) return;
      invoke("send_input", { sessionId, data: command + "\r" }).catch(() => {});
    },
    [sessionId],
  );

  if (!sessionId) {
    return (
      <div className="flex-1 flex items-center justify-center text-[var(--ui-fg-dim)] text-sm">
        No active session. Press Ctrl+T to create one.
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      className="flex-1 relative flex flex-col bg-[var(--color-bg)] focus:outline-none overflow-hidden"
      tabIndex={0}
      onKeyDown={handleKeyDown}
      onClick={handleClick}
    >
      {snapshot ? (
        <BlocksView snapshot={snapshot} onRerun={handleRerun} />
      ) : (
        <div className="flex-1 flex items-center justify-center text-[var(--color-text-muted)] text-sm">
          Loading...
        </div>
      )}
      <InputBar
        cwd={currentCwd}
        onSubmit={handleInputSubmit}
        onTab={handleInputTab}
        visible={!!sessionId && !exited}
      />
      <CompletionPopup />
      <SearchOverlay
        visible={searchVisible}
        onClose={handleSearchClose}
        onSearch={handleSearch}
        matchCount={searchMatches}
        currentMatch={searchCurrent}
        onNext={handleSearchNext}
        onPrevious={handleSearchPrevious}
      />
      {exited && (
        <div
          style={{
            padding: "var(--sp-3)",
            color: "var(--color-text-muted)",
            fontSize: 13,
            textAlign: "center",
          }}
        >
          [Process exited]
        </div>
      )}
    </div>
  );
}
