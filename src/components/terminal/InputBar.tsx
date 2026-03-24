import { useCallback, useRef, useEffect, useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTerminalStore } from "../../stores/terminalStore";
import { useCompletionStore, type CompletionItem } from "../../stores/completionStore";

interface ContextInfo {
  provider: string;
  data: Record<string, unknown>;
  detected_markers?: string[];
}

interface ProjectContext {
  project_root: string | null;
  cwd: string;
  providers: Record<string, ContextInfo>;
  last_updated: number;
  completion_sets: string[];
}

interface InputBarProps {
  cwd: string;
  sessionId: string;
  onSubmit: (input: string) => void;
  onTab: (input: string, cursorPos: number) => void;
  visible: boolean;
  agentMode?: boolean;
  agentName?: string;
  agentFile?: string;
}

export function InputBar({ cwd, sessionId, onSubmit, onTab, visible, agentMode, agentName, agentFile }: InputBarProps) {
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const [context, setContext] = useState<ProjectContext | null>(null);
  const [composing, setComposing] = useState(false);
  const [inputValue, setInputValue] = useState("");

  // Ghost text state
  const [ghostText, setGhostText] = useState("");
  const ghostFetchTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Command history
  const capturedBlocks = useTerminalStore((s) => s.capturedBlocks);
  const [historyIndex, setHistoryIndex] = useState(-1);
  const savedInputRef = useRef("");

  // Completion store
  const completionShow = useCompletionStore((s) => s.show);

  const history = useMemo(() => {
    const blocks = capturedBlocks.get(sessionId) ?? [];
    // Newest first, unique commands
    const seen = new Set<string>();
    const cmds: string[] = [];
    for (let i = blocks.length - 1; i >= 0; i--) {
      const cmd = blocks[i].command;
      if (!seen.has(cmd)) {
        seen.add(cmd);
        cmds.push(cmd);
      }
    }
    return cmds;
  }, [capturedBlocks, sessionId]);

  useEffect(() => {
    if (visible) {
      inputRef.current?.focus();
    }
  }, [visible]);

  // Fetch context when CWD changes
  useEffect(() => {
    if (!cwd) return;
    invoke<ProjectContext>("get_context", { cwd })
      .then(setContext)
      .catch(() => setContext(null));
  }, [cwd]);

  // Debounced ghost text fetch on input change (disabled in agent mode)
  const fetchGhostText = useCallback(
    (value: string) => {
      if (ghostFetchTimer.current) clearTimeout(ghostFetchTimer.current);
      if (agentMode || !value.trim()) {
        setGhostText("");
        return;
      }
      ghostFetchTimer.current = setTimeout(() => {
        const cursorPos = value.length;
        invoke<CompletionItem[]>("request_completions", {
          input: value,
          cursorPos,
          cwd,
        })
          .then((items) => {
            if (items.length > 0) {
              // Find ghost text: suffix of top match after current word
              const currentWord = getCurrentWord(value);
              const topText = items[0].text;
              if (topText.startsWith(currentWord) && topText.length > currentWord.length) {
                setGhostText(topText.slice(currentWord.length));
              } else {
                setGhostText("");
              }
            } else {
              setGhostText("");
            }
          })
          .catch(() => setGhostText(""));
      }, 150);
    },
    [cwd, agentMode],
  );

  const setInputAndSync = useCallback(
    (value: string) => {
      const el = inputRef.current;
      if (!el) return;
      el.value = value;
      el.style.height = "auto";
      el.style.height = Math.min(el.scrollHeight, 120) + "px";
      setInputValue(value);
      setGhostText("");
    },
    [],
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      e.stopPropagation();
      if (composing || e.nativeEvent.isComposing || e.key === "Process") return;

      const input = inputRef.current;
      if (!input) return;

      // Native clipboard/edit shortcuts
      if (e.ctrlKey && !e.shiftKey) {
        const k = e.key.toLowerCase();
        if (k === "c" || k === "v" || k === "a" || k === "z" || k === "x") {
          return;
        }
      }

      // Ctrl+Space → show completion dropdown
      if (e.ctrlKey && e.key === " ") {
        e.preventDefault();
        const value = input.value;
        const cursorPos = input.selectionStart ?? value.length;
        invoke<CompletionItem[]>("request_completions", {
          input: value,
          cursorPos,
          cwd,
        })
          .then((items) => {
            if (items.length > 0) completionShow(items);
          })
          .catch(() => {});
        return;
      }

      // Agent mode: Shift+Tab → send Shift+Tab escape sequence to PTY (accept)
      if (agentMode && e.key === "Tab" && e.shiftKey) {
        e.preventDefault();
        invoke("send_input", { sessionId, data: "\x1b[Z" }).catch(() => {});
        return;
      }

      // Agent mode: Escape → send Escape to PTY
      if (agentMode && e.key === "Escape") {
        e.preventDefault();
        invoke("send_input", { sessionId, data: "\x1b" }).catch(() => {});
        return;
      }

      // Enter → submit
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        const value = input.value;
        if (agentMode) {
          // In agent mode, send even empty lines (allows confirming prompts)
          onSubmit(value);
          input.value = "";
          input.style.height = "auto";
          setInputValue("");
          setGhostText("");
          setHistoryIndex(-1);
          savedInputRef.current = "";
          return;
        }
        if (value.trim()) {
          onSubmit(value);
          input.value = "";
          input.style.height = "auto";
          setInputValue("");
          setGhostText("");
          setHistoryIndex(-1);
          savedInputRef.current = "";
        }
        return;
      }

      // Tab → accept ghost text or trigger completion
      if (e.key === "Tab" && !e.shiftKey && !e.ctrlKey) {
        e.preventDefault();
        if (ghostText) {
          // Insert ghost text at cursor
          const value = input.value + ghostText;
          setInputAndSync(value);
          // Move cursor to end
          setTimeout(() => {
            input.selectionStart = input.selectionEnd = value.length;
          }, 0);
          fetchGhostText(value);
        } else {
          onTab(input.value, input.selectionStart ?? input.value.length);
        }
        return;
      }

      // Arrow Up → previous history
      if (e.key === "ArrowUp" && !e.ctrlKey && !e.shiftKey) {
        if (history.length === 0) return;
        e.preventDefault();
        if (historyIndex === -1) {
          savedInputRef.current = input.value;
        }
        const newIndex = Math.min(historyIndex + 1, history.length - 1);
        setHistoryIndex(newIndex);
        setInputAndSync(history[newIndex]);
        return;
      }

      // Arrow Down → next history / restore
      if (e.key === "ArrowDown" && !e.ctrlKey && !e.shiftKey) {
        if (historyIndex < 0) return;
        e.preventDefault();
        const newIndex = historyIndex - 1;
        setHistoryIndex(newIndex);
        if (newIndex < 0) {
          setInputAndSync(savedInputRef.current);
        } else {
          setInputAndSync(history[newIndex]);
        }
        return;
      }
    },
    [onSubmit, onTab, composing, ghostText, history, historyIndex, cwd, completionShow, fetchGhostText, setInputAndSync, agentMode, sessionId],
  );

  const handleInput = useCallback(() => {
    const el = inputRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = Math.min(el.scrollHeight, 120) + "px";
    setInputValue(el.value);
    setHistoryIndex(-1);
    fetchGhostText(el.value);
  }, [fetchGhostText]);

  const handleContainerClick = useCallback(() => {
    inputRef.current?.focus();
  }, []);

  const tokens = useMemo(() => tokenize(inputValue), [inputValue]);

  if (!visible) return null;

  // Extract context data
  const git = context?.providers?.git;
  const node = context?.providers?.node;
  const rust = context?.providers?.rust;
  const python = context?.providers?.python;

  const gitBranch = git?.data?.branch as string | undefined;
  const nodeRuntimeVersion = node?.data?.runtime_version as string | undefined;
  const rustRuntimeVersion = rust?.data?.runtime_version as string | undefined;
  const pythonRuntimeVersion = python?.data?.runtime_version as string | undefined;

  const modifiedCount = (git?.data?.modified_count as number) ?? 0;
  const stagedCount = (git?.data?.staged_count as number) ?? 0;
  const untrackedCount = (git?.data?.untracked_count as number) ?? 0;

  const runtimeVersion = nodeRuntimeVersion
    ? `v${nodeRuntimeVersion}`
    : rustRuntimeVersion
      ? `v${rustRuntimeVersion}`
      : pythonRuntimeVersion
        ? `v${pythonRuntimeVersion}`
        : null;

  const dirtyPlus = stagedCount;
  const dirtyMinus = modifiedCount + untrackedCount;

  return (
    <div
      style={{ padding: "8px 12px 12px 12px" }}
      className="shrink-0"
    >
      <div
        onClick={handleContainerClick}
        style={{
          background: "var(--color-surface)",
          border: "1px solid var(--color-border)",
          borderRadius: "var(--radius-xl)",
          overflow: "hidden",
          cursor: "text",
        }}
      >
        {/* Context badges bar (hidden in agent mode) */}
        {!agentMode && (
        <div
          style={{
            padding: "8px 14px 0 14px",
            fontSize: 11,
            fontFamily: "var(--font-mono)",
          }}
          className="flex items-center gap-1.5 flex-wrap"
        >
          {runtimeVersion && (
            <SquareBadge label={runtimeVersion} color="var(--color-success)" />
          )}
          <SquareBadge label={cwd || "~"} color="var(--color-text-muted)" />
          {gitBranch && (
            <SquareBadge label={`\u2387 ${gitBranch}`} color="var(--color-accent)" />
          )}
          {git && (
            <span
              className="shrink-0 flex items-center gap-1"
              style={{
                padding: "1px 6px",
                borderRadius: "var(--radius-sm)",
                background: "var(--color-surface-hover)",
                fontSize: 11,
                lineHeight: "18px",
                whiteSpace: "nowrap",
              }}
            >
              <span style={{ color: "var(--color-success)" }}>+{dirtyPlus}</span>
              <span style={{ color: "var(--color-error)" }}>-{dirtyMinus}</span>
            </span>
          )}
        </div>
        )}

        {/* Input area */}
        <div
          style={{ padding: "6px 14px 10px 14px" }}
          className="flex items-start gap-2"
        >
          <span
            style={{
              color: agentMode ? "var(--color-accent)" : "var(--color-primary)",
              fontSize: 14,
              fontFamily: "var(--font-mono)",
              fontWeight: 600,
              lineHeight: "24px",
              userSelect: "none",
            }}
          >
            {agentMode ? ">" : "$"}
          </span>
          <div style={{ position: "relative", flex: 1, minWidth: 0 }}>
            {/* Actual textarea */}
            <textarea
              ref={inputRef}
              onKeyDown={handleKeyDown}
              onInput={handleInput}
              onCompositionStart={() => setComposing(true)}
              onCompositionEnd={() => setComposing(false)}
              rows={1}
              style={{
                position: "relative",
                zIndex: 1,
                background: "transparent",
                border: "none",
                outline: "none",
                color: inputValue ? "transparent" : undefined,
                caretColor: "var(--color-text)",
                fontSize: 14,
                fontFamily: "var(--font-mono)",
                lineHeight: "24px",
                resize: "none",
                overflow: "hidden",
                width: "100%",
              }}
              placeholder={agentMode ? `Message ${agentName || "agent"}...` : "Type a command..."}
              spellCheck={false}
              autoComplete="off"
              autoCorrect="off"
            />
            {/* Syntax-highlighted overlay + ghost text */}
            <div
              aria-hidden
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                right: 0,
                zIndex: 0,
                pointerEvents: "none",
                whiteSpace: "pre-wrap",
                wordBreak: "break-all",
                fontSize: 14,
                fontFamily: "var(--font-mono)",
                lineHeight: "24px",
              }}
            >
              {inputValue ? (
                <>
                  {tokens.map((token, i) => (
                    <span key={i} style={{ color: TOKEN_COLORS[token.type] }}>
                      {token.text}
                    </span>
                  ))}
                  {ghostText && (
                    <span
                      style={{
                        color: "var(--color-text-muted)",
                        opacity: 0.4,
                      }}
                    >
                      {ghostText}
                    </span>
                  )}
                </>
              ) : null}
            </div>
          </div>
        </div>

        {/* Agent mode hint bar */}
        {agentMode && (
          <div
            style={{
              padding: "4px 14px 6px 14px",
              fontSize: 11,
              fontFamily: "var(--font-mono)",
              color: "var(--color-text-muted)",
              borderTop: "1px solid var(--color-border-muted)",
            }}
            className="flex items-center gap-3"
          >
            <span style={{ color: "var(--color-accent)", opacity: 0.7 }}>
              {agentName || "Agent"}
            </span>
            {agentFile && (
              <span style={{ opacity: 0.6 }}>
                In {agentFile}
              </span>
            )}
            <span style={{ marginLeft: "auto", opacity: 0.5 }}>
              Shift+Tab to accept
            </span>
          </div>
        )}
      </div>
    </div>
  );
}

/** Extract the current word at end of input (for ghost text matching). */
function getCurrentWord(input: string): string {
  const trimmed = input.trimEnd();
  if (!trimmed) return "";
  // Scan backward for space/tab
  let i = trimmed.length - 1;
  while (i >= 0 && trimmed[i] !== " " && trimmed[i] !== "\t") i--;
  return trimmed.slice(i + 1);
}

// --- Square badge component ---

function SquareBadge({
  label,
  color,
  title,
}: {
  label: string;
  color: string;
  title?: string;
}) {
  return (
    <span
      className="shrink-0"
      style={{
        padding: "1px 6px",
        borderRadius: "var(--radius-sm)",
        background: "var(--color-surface-hover)",
        color,
        fontSize: 11,
        lineHeight: "18px",
        whiteSpace: "nowrap",
      }}
      title={title}
    >
      {label}
    </span>
  );
}

// --- Syntax highlighting ---

type TokenType = "command" | "flag" | "argument" | "pipe" | "redirect" | "string";

interface Token {
  text: string;
  type: TokenType;
}

const TOKEN_COLORS: Record<TokenType, string> = {
  command: "var(--color-info)",
  flag: "var(--color-accent)",
  argument: "var(--color-text)",
  pipe: "var(--color-text-muted)",
  redirect: "var(--color-text-muted)",
  string: "var(--color-success)",
};

function tokenize(input: string): Token[] {
  if (!input) return [];

  const tokens: Token[] = [];
  let i = 0;
  let expectCommand = true;

  while (i < input.length) {
    if (input[i] === " " || input[i] === "\t") {
      const start = i;
      while (i < input.length && (input[i] === " " || input[i] === "\t")) i++;
      tokens.push({ text: input.slice(start, i), type: "argument" });
      continue;
    }

    if (input[i] === "|" || input[i] === "&" || input[i] === ";") {
      let op = input[i];
      if (i + 1 < input.length && input[i + 1] === input[i]) {
        op += input[i + 1];
        i += 2;
      } else {
        i++;
      }
      tokens.push({ text: op, type: "pipe" });
      expectCommand = true;
      continue;
    }

    if (input[i] === ">" || input[i] === "<") {
      let op = input[i];
      if (i + 1 < input.length && input[i + 1] === input[i]) {
        op += input[i + 1];
        i += 2;
      } else {
        i++;
      }
      tokens.push({ text: op, type: "redirect" });
      continue;
    }

    if (input[i] === '"' || input[i] === "'") {
      const quote = input[i];
      const start = i;
      i++;
      while (i < input.length && input[i] !== quote) {
        if (input[i] === "\\" && quote === '"') i++;
        i++;
      }
      if (i < input.length) i++;
      tokens.push({ text: input.slice(start, i), type: "string" });
      if (expectCommand) expectCommand = false;
      continue;
    }

    const start = i;
    while (
      i < input.length &&
      input[i] !== " " &&
      input[i] !== "\t" &&
      input[i] !== "|" &&
      input[i] !== "&" &&
      input[i] !== ";" &&
      input[i] !== ">" &&
      input[i] !== "<" &&
      input[i] !== '"' &&
      input[i] !== "'"
    ) {
      i++;
    }
    const word = input.slice(start, i);

    if (expectCommand) {
      tokens.push({ text: word, type: "command" });
      expectCommand = false;
    } else if (word.startsWith("-")) {
      tokens.push({ text: word, type: "flag" });
    } else {
      tokens.push({ text: word, type: "argument" });
    }
  }

  return tokens;
}
