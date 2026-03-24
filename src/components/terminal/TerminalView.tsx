import { useEffect, useRef, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { BlocksView } from "./BlocksView";
import { InputBar } from "./InputBar";
import { CompletionPopup } from "./CompletionPopup";
import { SearchOverlay, type SearchOptions } from "./SearchOverlay";
import {
  useTerminalStore,
  getNextCommandId,
  type GridSnapshot,
} from "../../stores/terminalStore";
import { useCompletionStore, type CompletionItem } from "../../stores/completionStore";
import { useSessionStore } from "../../stores/sessionStore";
import { useContextStore } from "../../stores/contextStore";
import { useAgentStore } from "../../stores/agentStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useInputBuffer } from "../../hooks/useInputBuffer";

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

interface CommandOutputPayload {
  session_id: string;
  command_id: number;
  output: string;
  duration_ms: number;
}

interface CommandOutputChunkPayload {
  session_id: string;
  command_id: number;
  output: string;
}

interface AgentDetectedPayload {
  session_id: string;
  agent_name: string;
  agent_kind: string;
  pid: number;
}

interface AgentExitedPayload {
  session_id: string;
  agent_name: string;
}

interface AgentEventPayload {
  session_id: string;
  event_type: string;
  data: Record<string, unknown>;
  timestamp: number;
}

interface FileChangePayload {
  session_id: string;
  path: string;
  action: "created" | "modified" | "deleted";
  timestamp: number;
}

let agentEventCounter = 0;

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

  const selectedBlockIndex = useTerminalStore((s) => s.selectedBlockIndex);
  const selectBlock = useTerminalStore((s) => s.selectBlock);
  const moveBlockSelection = useTerminalStore((s) => s.moveBlockSelection);
  const addCapturedBlock = useTerminalStore((s) => s.addCapturedBlock);
  const updateOutputChunk = useTerminalStore((s) => s.updateOutputChunk);
  const finalizeOutput = useTerminalStore((s) => s.finalizeOutput);
  const capturedBlocks = useTerminalStore((s) => s.capturedBlocks);

  const { append, reset, getBuffer, getCursor, insertCompletion } =
    useInputBuffer();

  const snapshot = sessionId ? grids.get(sessionId) : undefined;
  const exited = sessionId ? exitedSessions.has(sessionId) : false;

  // Sync CWD from session store (for initial CWD before any cwd_changed event)
  const sessionCwd = useSessionStore(
    (s) => s.sessions.find((sess) => sess.id === sessionId)?.cwd ?? "",
  );
  const effectiveCwd = currentCwd || sessionCwd;

  // Keep cwdRef in sync with effectiveCwd
  useEffect(() => {
    if (effectiveCwd) {
      cwdRef.current = effectiveCwd;
    }
  }, [effectiveCwd]);

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

    const unlisten5 = listen<CommandOutputPayload>(
      "command_output",
      (event) => {
        finalizeOutput(
          event.payload.session_id,
          event.payload.command_id,
          event.payload.output,
          event.payload.duration_ms,
        );
      },
    );

    let idleTimer: ReturnType<typeof setTimeout> | null = null;

    const unlisten6 = listen<CommandOutputChunkPayload>(
      "command_output_chunk",
      (event) => {
        const { session_id: sid, command_id: cmdId, output } = event.payload;
        updateOutputChunk(sid, cmdId, output);

        // Auto-finalize after 500ms of no new chunks
        if (idleTimer) clearTimeout(idleTimer);
        const chunkReceivedAt = Date.now();
        idleTimer = setTimeout(() => {
          const blocks = useTerminalStore.getState().capturedBlocks.get(sid);
          const block = blocks?.find((b) => b.id === cmdId);
          if (block && block.durationMs == null) {
            // Duration = last actual output time - submit time (not including idle wait)
            const duration = chunkReceivedAt - block.submittedAt;
            finalizeOutput(sid, cmdId, output, duration);
          }
        }, 500);
      },
    );

    return () => {
      if (idleTimer) clearTimeout(idleTimer);
      unlisten1.then((fn) => fn());
      unlisten2.then((fn) => fn());
      unlisten3.then((fn) => fn());
      unlisten4.then((fn) => fn());
      unlisten5.then((fn) => fn());
      unlisten6.then((fn) => fn());
    };
  }, [updateGrid, updateSessionCwd, updateSessionTitle, sessionId, reset, finalizeOutput, updateOutputChunk]);

  // Listen for agent events
  useEffect(() => {
    const {
      setAgent,
      endAgent,
      addEvent,
      addFileChange,
      updateTokens,
      updateCost,
      updateModel,
      updateFile,
      setThinking,
    } = useAgentStore.getState();

    const unlisten7 = listen<AgentDetectedPayload>("agent_detected", (event) => {
      const p = event.payload;
      setAgent(p.session_id, {
        name: p.agent_name,
        kind: p.agent_kind,
        pid: p.pid,
        detectedAt: Date.now(),
      });
    });

    const unlisten8 = listen<AgentExitedPayload>("agent_exited", (event) => {
      endAgent(event.payload.session_id);
    });

    const unlisten9 = listen<AgentEventPayload>("agent_event", (event) => {
      const p = event.payload;
      const eventId = `agent-evt-${++agentEventCounter}`;
      const timelineEvent = {
        id: eventId,
        eventType: p.event_type,
        data: p.data,
        timestamp: p.timestamp,
      };
      addEvent(p.session_id, timelineEvent);

      switch (p.event_type) {
        case "thinking_start":
          setThinking(p.session_id, true);
          break;
        case "thinking_end":
          setThinking(p.session_id, false);
          break;
        case "token_update":
          updateTokens(
            p.session_id,
            Number(p.data.input ?? 0),
            Number(p.data.output ?? 0),
          );
          break;
        case "cost_update":
          updateCost(p.session_id, Number(p.data.total_cost ?? 0));
          break;
        case "model_info":
          updateModel(p.session_id, String(p.data.model_name ?? ""));
          break;
        case "file_edit":
          updateFile(p.session_id, String(p.data.path ?? ""));
          break;
      }
    });

    const unlisten10 = listen<FileChangePayload>("file_change", (event) => {
      const p = event.payload;
      addFileChange(p.session_id, {
        path: p.path,
        action: p.action,
        timestamp: p.timestamp,
      });
    });

    return () => {
      unlisten7.then((fn) => fn());
      unlisten8.then((fn) => fn());
      unlisten9.then((fn) => fn());
      unlisten10.then((fn) => fn());
    };
  }, []);

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

  const focusInputBar = useCallback(() => {
    const textarea = containerRef.current?.querySelector<HTMLTextAreaElement>(
      "textarea[placeholder]",
    );
    textarea?.focus();
  }, []);

  // Handle keyboard input
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (!sessionId || exited) return;

      if (e.nativeEvent.isComposing || e.key === "Process") return;

      const sessions = useSessionStore.getState().sessions;

      if (e.ctrlKey && !e.shiftKey && (e.key === "t" || e.key === "T")) {
        e.preventDefault();
        const rect = containerRef.current?.getBoundingClientRect();
        const cols = rect ? Math.max(1, Math.floor(rect.width / 8.4)) : undefined;
        const rows = rect ? Math.max(1, Math.floor(rect.height / (14 * 1.2))) : undefined;
        useSessionStore.getState().createNewSession(undefined, cols, rows);
        return;
      }

      if (e.ctrlKey && !e.shiftKey && (e.key === "w" || e.key === "W")) {
        e.preventDefault();
        useSessionStore.getState().closeSession(sessionId);
        return;
      }

      if (e.ctrlKey && e.key === "Tab") {
        e.preventDefault();
        if (e.shiftKey) {
          useSessionStore.getState().switchToPrevious();
        } else {
          useSessionStore.getState().switchToNext();
        }
        return;
      }

      if (e.ctrlKey && !e.shiftKey && e.key >= "1" && e.key <= "9") {
        const idx = parseInt(e.key) - 1;
        if (idx < sessions.length) {
          e.preventDefault();
          useSessionStore.getState().switchToIndex(idx);
          return;
        }
      }

      const blockCount = snapshot?.blocks.length ?? 0;
      if (e.ctrlKey && !e.shiftKey && e.key === "ArrowUp") {
        e.preventDefault();
        moveBlockSelection("up", blockCount);
        return;
      }
      if (e.ctrlKey && !e.shiftKey && e.key === "ArrowDown") {
        e.preventDefault();
        moveBlockSelection("down", blockCount);
        return;
      }
      if (selectedBlockIndex !== null && !completionVisible) {
        if (e.key === "ArrowUp") {
          e.preventDefault();
          moveBlockSelection("up", blockCount);
          return;
        }
        if (e.key === "ArrowDown") {
          e.preventDefault();
          moveBlockSelection("down", blockCount);
          return;
        }
        if (e.key === "Escape") {
          e.preventDefault();
          selectBlock(null);
          return;
        }
        selectBlock(null);
      }

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

      if (e.ctrlKey && e.shiftKey && (e.key === "F" || e.key === "f")) {
        e.preventDefault();
        setSearchVisible((v) => !v);
        return;
      }

      if (e.ctrlKey && !e.shiftKey && (e.key === "c" || e.key === "C")) {
        const selection = window.getSelection()?.toString();
        if (selection) {
          e.preventDefault();
          navigator.clipboard.writeText(selection).catch(() => {});
          return;
        }
        // Agent mode: Ctrl+C with no selection sends SIGINT to PTY
        if (isAgentActive) {
          e.preventDefault();
          invoke("send_input", { sessionId, data: "\x03" }).catch(() => {});
          return;
        }
      }

      if (e.ctrlKey && !e.shiftKey && (e.key === "v" || e.key === "V")) {
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

      if (e.ctrlKey && e.shiftKey && (e.key === "C" || e.key === "c")) {
        e.preventDefault();
        const selection = window.getSelection()?.toString();
        if (selection) {
          navigator.clipboard.writeText(selection).catch(() => {});
        }
        return;
      }

      focusInputBar();
    },
    [
      sessionId,
      exited,
      snapshot,
      completionVisible,
      completionSelectNext,
      completionSelectPrevious,
      completionGetSelected,
      completionHide,
      requestCompletions,
      insertCompletion,
      append,
      selectedBlockIndex,
      selectBlock,
      moveBlockSelection,
      focusInputBar,
    ],
  );

  const handleClick = useCallback(() => {
    const textarea = containerRef.current?.querySelector<HTMLTextAreaElement>(
      "textarea[placeholder]",
    );
    textarea?.focus();
  }, []);

  // Agent mode detection
  const agentSession = useAgentStore(
    (s) => (sessionId ? s.sessions[sessionId] : undefined),
  );
  const isAgentActive = !!(agentSession && !agentSession.isEnded);
  const agentName = agentSession?.identity?.name;
  const agentFile = agentSession?.currentFile;
  const filterChrome = useSettingsStore((s) => s.config.agent_filter_chrome);

  const handleInputSubmit = useCallback(
    (input: string) => {
      if (!sessionId) return;

      if (isAgentActive) {
        // Agent mode: send directly to PTY stdin (chat with agent)
        invoke("send_input", { sessionId, data: input + "\n" }).catch(() => {});
        return;
      }

      const commandId = getNextCommandId();
      const ctx = useContextStore.getState().context;
      const gitBranch = ctx?.providers?.git?.data?.branch as string | undefined;
      // Add block to store
      addCapturedBlock(sessionId, commandId, input, cwdRef.current, gitBranch);
      // Submit via Rust (atomically captures + writes to PTY)
      invoke("submit_command", { sessionId, command: input, commandId }).catch(() => {});
    },
    [sessionId, addCapturedBlock, isAgentActive],
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
      // Re-run goes through submit_command too
      const commandId = getNextCommandId();
      addCapturedBlock(sessionId, commandId, command, cwdRef.current);
      invoke("submit_command", { sessionId, command, commandId }).catch(() => {});
    },
    [sessionId, addCapturedBlock],
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
      className="flex-1 relative flex flex-col focus:outline-none overflow-hidden"
      style={{ background: "var(--term-bg)" }}
      tabIndex={0}
      onKeyDown={handleKeyDown}
      onClick={handleClick}
    >
      {snapshot ? (
        <BlocksView
          snapshot={snapshot}
          onRerun={handleRerun}
          selectedBlockIndex={selectedBlockIndex}
          onSelectBlock={selectBlock}
          capturedBlocks={sessionId ? capturedBlocks.get(sessionId) : undefined}
          agentMode={isAgentActive}
          filterChrome={filterChrome}
        />
      ) : (
        <div className="flex-1 flex items-center justify-center text-[var(--color-text-muted)] text-sm">
          Loading...
        </div>
      )}
      <InputBar
        cwd={effectiveCwd}
        sessionId={sessionId}
        onSubmit={handleInputSubmit}
        onTab={handleInputTab}
        visible={!!sessionId && !exited}
        agentMode={isAgentActive}
        agentName={agentName}
        agentFile={agentFile}
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
