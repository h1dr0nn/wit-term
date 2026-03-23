import { useCallback, useRef, useEffect } from "react";

interface InputBarProps {
  cwd: string;
  onSubmit: (input: string) => void;
  onTab: (input: string, cursorPos: number) => void;
  visible: boolean;
}

/**
 * Warp-style input bar at the bottom of the terminal.
 * Shows current directory above the input field.
 */
export function InputBar({ cwd, onSubmit, onTab, visible }: InputBarProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (visible) {
      inputRef.current?.focus();
    }
  }, [visible]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      const input = inputRef.current;
      if (!input) return;

      if (e.key === "Enter") {
        e.preventDefault();
        const value = input.value;
        if (value.trim()) {
          onSubmit(value);
          input.value = "";
        }
      }

      if (e.key === "Tab" && !e.shiftKey && !e.ctrlKey) {
        e.preventDefault();
        onTab(input.value, input.selectionStart ?? input.value.length);
      }
    },
    [onSubmit, onTab],
  );

  if (!visible) return null;

  const cwdDisplay = cwdShort(cwd);

  return (
    <div
      style={{
        borderTop: "1px solid var(--color-border-muted)",
        background: "var(--color-surface)",
      }}
      className="shrink-0"
    >
      {/* Address bar - CWD */}
      <div
        style={{
          padding: "6px 12px 2px 12px",
          fontSize: 12,
          color: "var(--color-text-muted)",
          fontFamily: "var(--font-mono)",
        }}
        className="flex items-center gap-1.5"
      >
        {/* Folder icon */}
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.2">
          <path d="M1.5 3.5C1.5 2.95 1.95 2.5 2.5 2.5H4.5L5.5 3.5H9.5C10.05 3.5 10.5 3.95 10.5 4.5V8.5C10.5 9.05 10.05 9.5 9.5 9.5H2.5C1.95 9.5 1.5 9.05 1.5 8.5V3.5Z" />
        </svg>
        <span className="truncate" title={cwd}>
          {cwdDisplay}
        </span>
      </div>

      {/* Input field */}
      <div
        style={{
          padding: "4px 12px 8px 12px",
        }}
        className="flex items-center gap-2"
      >
        {/* Prompt indicator */}
        <span
          style={{
            color: "var(--color-primary)",
            fontSize: 14,
            fontFamily: "var(--font-mono)",
            fontWeight: 600,
          }}
        >
          $
        </span>
        <input
          ref={inputRef}
          type="text"
          onKeyDown={handleKeyDown}
          style={{
            background: "transparent",
            border: "none",
            outline: "none",
            color: "var(--color-text)",
            fontSize: 14,
            fontFamily: "var(--font-mono)",
            lineHeight: "22px",
          }}
          className="flex-1 min-w-0"
          placeholder="Type a command..."
          spellCheck={false}
          autoComplete="off"
          autoCorrect="off"
        />
      </div>
    </div>
  );
}

function cwdShort(cwd: string): string {
  if (!cwd) return "~";
  const normalized = cwd.replace(/\\/g, "/");
  const home = process.env.HOME || process.env.USERPROFILE || "";
  if (home && normalized.startsWith(home.replace(/\\/g, "/"))) {
    return "~" + normalized.slice(home.length);
  }
  const parts = normalized.split("/").filter(Boolean);
  if (parts.length <= 3) return normalized;
  return ".../" + parts.slice(-3).join("/");
}
