import { useCallback, useRef, useEffect, useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";

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
  onSubmit: (input: string) => void;
  onTab: (input: string, cursorPos: number) => void;
  visible: boolean;
}

export function InputBar({ cwd, onSubmit, onTab, visible }: InputBarProps) {
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const [context, setContext] = useState<ProjectContext | null>(null);
  const [composing, setComposing] = useState(false);
  const [inputValue, setInputValue] = useState("");

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

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      // Stop events from reaching TerminalView's handler
      e.stopPropagation();

      // Skip during IME composition (Vietnamese, Chinese, etc.)
      if (composing || e.nativeEvent.isComposing || e.key === "Process") return;

      const input = inputRef.current;
      if (!input) return;

      // Let browser handle native clipboard/edit shortcuts
      if (e.ctrlKey && !e.shiftKey) {
        const k = e.key.toLowerCase();
        if (k === "c" || k === "v" || k === "a" || k === "z" || k === "x") {
          return; // Native copy/paste/select-all/undo/cut
        }
      }

      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        const value = input.value;
        if (value.trim()) {
          onSubmit(value);
          input.value = "";
          input.style.height = "auto";
          setInputValue("");
        }
      }

      if (e.key === "Tab" && !e.shiftKey && !e.ctrlKey) {
        e.preventDefault();
        onTab(input.value, input.selectionStart ?? input.value.length);
      }
    },
    [onSubmit, onTab, composing],
  );

  const handleInput = useCallback(() => {
    const el = inputRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = Math.min(el.scrollHeight, 120) + "px";
    setInputValue(el.value);
  }, []);

  // Click anywhere on the bar to focus the input
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
  const nodeVersion = node?.data?.version as string | undefined;
  const rustVersion = rust?.data?.version as string | undefined;
  const pythonVersion = python?.data?.version as string | undefined;

  // Total changes count (modified + staged + untracked)
  const modifiedCount = (git?.data?.modified_count as number) ?? 0;
  const stagedCount = (git?.data?.staged_count as number) ?? 0;
  const untrackedCount = (git?.data?.untracked_count as number) ?? 0;
  const totalChanges = modifiedCount + stagedCount + untrackedCount;

  // Runtime/tool version display
  const runtimeVersion = nodeVersion
    ? `v${nodeVersion}`
    : rustVersion
      ? `v${rustVersion}`
      : pythonVersion
        ? `v${pythonVersion}`
        : null;

  const cwdDisplay = cwdShort(cwd);

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
        {/* Context badges bar */}
        <div
          style={{
            padding: "8px 14px 0 14px",
            fontSize: 11,
            fontFamily: "var(--font-mono)",
          }}
          className="flex items-center gap-1.5 flex-wrap"
        >
          {/* Runtime/Tool Version */}
          {runtimeVersion && (
            <SquareBadge label={runtimeVersion} color="var(--color-success)" />
          )}

          {/* Workspace / Project Path */}
          <SquareBadge label={cwdDisplay} color="var(--color-text-muted)" title={cwd} />

          {/* Git Branch */}
          {gitBranch && (
            <SquareBadge label={gitBranch} color="var(--color-accent)" />
          )}

          {/* Git Status (changes count) */}
          {git && (
            <SquareBadge
              label={String(totalChanges)}
              color={totalChanges === 0 ? "var(--color-success)" : "var(--color-warning)"}
            />
          )}
        </div>

        {/* Input area */}
        <div
          style={{ padding: "6px 14px 10px 14px" }}
          className="flex items-start gap-2"
        >
          <span
            style={{
              color: "var(--color-primary)",
              fontSize: 14,
              fontFamily: "var(--font-mono)",
              fontWeight: 600,
              lineHeight: "24px",
              userSelect: "none",
            }}
          >
            $
          </span>
          <div style={{ position: "relative", flex: 1, minWidth: 0 }}>
            {/* Actual textarea - on top for interaction, transparent text */}
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
              placeholder="Type a command..."
              spellCheck={false}
              autoComplete="off"
              autoCorrect="off"
            />
            {/* Syntax-highlighted overlay - behind textarea, visible through transparent text */}
            {inputValue && (
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
                {tokens.map((token, i) => (
                  <span key={i} style={{ color: TOKEN_COLORS[token.type] }}>
                    {token.text}
                  </span>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

// --- Square badge component: [label] ---

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
  let expectCommand = true; // first word or after pipe/redirect

  while (i < input.length) {
    // Whitespace
    if (input[i] === " " || input[i] === "\t") {
      const start = i;
      while (i < input.length && (input[i] === " " || input[i] === "\t")) i++;
      tokens.push({ text: input.slice(start, i), type: "argument" });
      continue;
    }

    // Pipe/redirect operators (multi-char: ||, &&, >>)
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

    // Quoted strings
    if (input[i] === '"' || input[i] === "'") {
      const quote = input[i];
      const start = i;
      i++; // skip opening quote
      while (i < input.length && input[i] !== quote) {
        if (input[i] === "\\" && quote === '"') i++; // skip escaped char
        i++;
      }
      if (i < input.length) i++; // skip closing quote
      tokens.push({ text: input.slice(start, i), type: "string" });
      if (expectCommand) expectCommand = false;
      continue;
    }

    // Regular word
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

// --- Helpers ---

function cwdShort(cwd: string): string {
  if (!cwd) return "~";
  const normalized = cwd.replace(/\\/g, "/");
  const parts = normalized.split("/").filter(Boolean);
  if (parts.length <= 3) return normalized;
  return ".../" + parts.slice(-2).join("/");
}
