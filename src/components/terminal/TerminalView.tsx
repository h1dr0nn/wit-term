import { useEffect, useRef, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { TerminalGrid } from "./TerminalGrid";
import { useTerminalStore, type GridSnapshot } from "../../stores/terminalStore";
import { encodeKey } from "../../utils/keyEncoder";

interface GridUpdatePayload {
  session_id: string;
  snapshot: GridSnapshot;
}

interface SessionExitedPayload {
  session_id: string;
  exit_code: number;
}

export function TerminalView() {
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [exited, setExited] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const updateGrid = useTerminalStore((s) => s.updateGrid);
  const grids = useTerminalStore((s) => s.grids);

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

  // Listen for grid updates
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

    return () => {
      unlisten.then((fn) => fn());
      unlistenExit.then((fn) => fn());
    };
  }, [sessionId, updateGrid]);

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

  // Handle keyboard input
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (!sessionId || exited) return;

      // Clipboard: Ctrl+Shift+V = paste
      if (e.ctrlKey && e.shiftKey && (e.key === "V" || e.key === "v")) {
        e.preventDefault();
        navigator.clipboard
          .readText()
          .then((text) => {
            if (text) {
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
        invoke("send_input", { sessionId, data }).catch((err) => {
          console.error("Failed to send input:", err);
        });
      }
    },
    [sessionId, exited],
  );

  const handleClick = useCallback(() => {
    containerRef.current?.focus();
  }, []);

  return (
    <div
      ref={containerRef}
      className="flex-1 flex flex-col bg-[#1e1e2e] focus:outline-none overflow-hidden p-1"
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
      {exited && (
        <div className="text-[#a6adc8] text-sm p-2">[Process exited]</div>
      )}
    </div>
  );
}
