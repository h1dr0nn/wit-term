import { useEffect, useRef, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { TerminalGrid } from "./TerminalGrid";
import { CompletionPopup } from "./CompletionPopup";
import { useTerminalStore, type GridSnapshot } from "../../stores/terminalStore";
import { useCompletionStore, type CompletionItem } from "../../stores/completionStore";
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

export function TerminalView() {
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [exited, setExited] = useState(false);
  const cwdRef = useRef<string>("");
  const containerRef = useRef<HTMLDivElement>(null);
  const updateGrid = useTerminalStore((s) => s.updateGrid);
  const grids = useTerminalStore((s) => s.grids);

  const completionVisible = useCompletionStore((s) => s.visible);
  const completionShow = useCompletionStore((s) => s.show);
  const completionHide = useCompletionStore((s) => s.hide);
  const completionSelectNext = useCompletionStore((s) => s.selectNext);
  const completionSelectPrevious = useCompletionStore((s) => s.selectPrevious);
  const completionGetSelected = useCompletionStore((s) => s.getSelected);

  const { append, reset, getBuffer, getCursor, insertCompletion } = useInputBuffer();

  const snapshot = sessionId ? grids.get(sessionId) : undefined;

  // Create session on mount
  useEffect(() => {
    let cancelled = false;

    invoke<string>("create_session")
      .then((id) => {
        if (!cancelled) {
          setSessionId(id);
          containerRef.current?.focus();
        }
      })
      .catch((err) => {
        console.error("Failed to create session:", err);
      });

    return () => {
      cancelled = true;
    };
  }, []);

  // Listen for events
  useEffect(() => {
    if (!sessionId) return;

    const unlisten = listen<GridUpdatePayload>("grid_update", (event) => {
      if (event.payload.session_id === sessionId) {
        updateGrid(sessionId, event.payload.snapshot);
      }
    });

    const unlistenExit = listen<SessionExitedPayload>("session_exited", (event) => {
      if (event.payload.session_id === sessionId) {
        setExited(true);
      }
    });

    const unlistenCwd = listen<CwdChangedPayload>("cwd_changed", (event) => {
      if (event.payload.session_id === sessionId) {
        cwdRef.current = event.payload.cwd;
        // CWD change means new prompt — reset input buffer
        reset();
      }
    });

    return () => {
      unlisten.then((fn) => fn());
      unlistenExit.then((fn) => fn());
      unlistenCwd.then((fn) => fn());
    };
  }, [sessionId, updateGrid, reset]);

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
        // Single match: auto-insert directly
        const data = insertCompletion(items[0].text);
        if (sessionId && data) {
          await invoke("send_input", { sessionId, data });
        }
      } else if (items.length > 1) {
        completionShow(items);
      }
    } catch {
      // Completion request failed — ignore silently
    }
  }, [getBuffer, getCursor, insertCompletion, sessionId, completionShow]);

  // Handle keyboard input
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (!sessionId || exited) return;

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

      // Tab key (when popup not visible) → trigger completions
      if (e.key === "Tab" && !e.shiftKey && !e.ctrlKey && !e.altKey && !e.metaKey) {
        e.preventDefault();
        requestCompletions();
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
        // Track in input buffer
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

  return (
    <div
      ref={containerRef}
      className="flex-1 relative flex flex-col bg-[#1e1e2e] focus:outline-none overflow-hidden p-1"
      tabIndex={0}
      onKeyDown={handleKeyDown}
      onClick={handleClick}
    >
      {snapshot ? (
        <TerminalGrid snapshot={snapshot} />
      ) : (
        <div className="flex-1 flex items-center justify-center text-[#a6adc8] text-sm">
          {sessionId ? "Loading..." : "Connecting..."}
        </div>
      )}
      <CompletionPopup />
      {exited && (
        <div className="text-[#a6adc8] text-sm p-2">[Process exited]</div>
      )}
    </div>
  );
}
