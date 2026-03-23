import { useCallback, useRef, useEffect } from "react";
import { Folder, ChevronRight } from "lucide-react";

interface InputBarProps {
  cwd: string;
  onSubmit: (input: string) => void;
  onTab: (input: string, cursorPos: number) => void;
  visible: boolean;
}

export function InputBar({ cwd, onSubmit, onTab, visible }: InputBarProps) {
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (visible) {
      inputRef.current?.focus();
    }
  }, [visible]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      const input = inputRef.current;
      if (!input) return;

      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        const value = input.value;
        if (value.trim()) {
          onSubmit(value);
          input.value = "";
          // Reset textarea height
          input.style.height = "auto";
        }
      }

      if (e.key === "Tab" && !e.shiftKey && !e.ctrlKey) {
        e.preventDefault();
        onTab(input.value, input.selectionStart ?? input.value.length);
      }
    },
    [onSubmit, onTab],
  );

  // Auto-resize textarea
  const handleInput = useCallback(() => {
    const el = inputRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = Math.min(el.scrollHeight, 120) + "px";
  }, []);

  if (!visible) return null;

  const cwdDisplay = cwdShort(cwd);

  return (
    <div
      style={{ padding: "8px 12px 12px 12px" }}
      className="shrink-0"
    >
      <div
        style={{
          background: "var(--color-surface)",
          border: "1px solid var(--color-border)",
          borderRadius: "var(--radius-xl)",
          overflow: "hidden",
        }}
      >
        {/* CWD address bar */}
        <div
          style={{
            padding: "8px 14px 0 14px",
            fontSize: 12,
            color: "var(--color-text-muted)",
            fontFamily: "var(--font-mono)",
          }}
          className="flex items-center gap-1.5"
        >
          <Folder size={12} strokeWidth={2} className="shrink-0" />
          <span className="truncate" title={cwd}>
            {cwdDisplay}
          </span>
          <ChevronRight size={10} strokeWidth={2} className="shrink-0 opacity-50" />
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
          <textarea
            ref={inputRef}
            onKeyDown={handleKeyDown}
            onInput={handleInput}
            rows={1}
            style={{
              background: "transparent",
              border: "none",
              outline: "none",
              color: "var(--color-text)",
              fontSize: 14,
              fontFamily: "var(--font-mono)",
              lineHeight: "24px",
              resize: "none",
              overflow: "hidden",
            }}
            className="flex-1 min-w-0"
            placeholder="Type a command..."
            spellCheck={false}
            autoComplete="off"
            autoCorrect="off"
          />
        </div>
      </div>
    </div>
  );
}

function cwdShort(cwd: string): string {
  if (!cwd) return "~";
  const normalized = cwd.replace(/\\/g, "/");
  // Try to shorten home dir
  const parts = normalized.split("/").filter(Boolean);
  if (parts.length <= 3) return normalized;
  return "~/" + parts.slice(-2).join("/");
}
