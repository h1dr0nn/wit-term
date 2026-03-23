import { useState, useCallback, useRef, useEffect } from "react";

interface SearchOverlayProps {
  visible: boolean;
  onClose: () => void;
  onSearch: (query: string, options: SearchOptions) => void;
  matchCount: number;
  currentMatch: number;
  onNext: () => void;
  onPrevious: () => void;
}

export interface SearchOptions {
  caseSensitive: boolean;
  regex: boolean;
}

export function SearchOverlay({
  visible,
  onClose,
  onSearch,
  matchCount,
  currentMatch,
  onNext,
  onPrevious,
}: SearchOverlayProps) {
  const [query, setQuery] = useState("");
  const [caseSensitive, setCaseSensitive] = useState(false);
  const [regex, setRegex] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (visible) {
      inputRef.current?.focus();
      inputRef.current?.select();
    }
  }, [visible]);

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = e.target.value;
      setQuery(value);
      onSearch(value, { caseSensitive, regex });
    },
    [onSearch, caseSensitive, regex],
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        onClose();
      } else if (e.key === "Enter") {
        e.preventDefault();
        if (e.shiftKey) {
          onPrevious();
        } else {
          onNext();
        }
      }
    },
    [onClose, onNext, onPrevious],
  );

  const toggleCaseSensitive = useCallback(() => {
    const next = !caseSensitive;
    setCaseSensitive(next);
    onSearch(query, { caseSensitive: next, regex });
  }, [caseSensitive, query, onSearch, regex]);

  const toggleRegex = useCallback(() => {
    const next = !regex;
    setRegex(next);
    onSearch(query, { caseSensitive, regex: next });
  }, [regex, query, onSearch, caseSensitive]);

  if (!visible) return null;

  return (
    <div className="absolute top-2 right-2 z-50 flex items-center gap-1 rounded border border-[var(--ui-border)] bg-[var(--ui-bg-tertiary)] px-2 py-1.5 shadow-lg">
      <input
        ref={inputRef}
        type="text"
        value={query}
        onChange={handleChange}
        onKeyDown={handleKeyDown}
        className="bg-transparent border-none outline-none text-[var(--ui-fg)] text-sm w-48 placeholder:text-[var(--ui-fg-dim)]"
        placeholder="Search..."
      />
      <span className="text-xs text-[var(--ui-fg-dim)] min-w-12 text-center">
        {matchCount > 0 ? `${currentMatch + 1}/${matchCount}` : "No results"}
      </span>
      <button
        onClick={toggleCaseSensitive}
        className={`px-1.5 py-0.5 text-xs rounded ${
          caseSensitive
            ? "bg-[var(--ui-accent)] text-[var(--ui-bg)]"
            : "text-[var(--ui-fg-dim)] hover:text-[var(--ui-fg)]"
        }`}
        title="Match case"
      >
        Aa
      </button>
      <button
        onClick={toggleRegex}
        className={`px-1.5 py-0.5 text-xs rounded ${
          regex
            ? "bg-[var(--ui-accent)] text-[var(--ui-bg)]"
            : "text-[var(--ui-fg-dim)] hover:text-[var(--ui-fg)]"
        }`}
        title="Regex"
      >
        .*
      </button>
      <button
        onClick={onPrevious}
        className="text-[var(--ui-fg-dim)] hover:text-[var(--ui-fg)] px-1"
        title="Previous (Shift+Enter)"
      >
        ^
      </button>
      <button
        onClick={onNext}
        className="text-[var(--ui-fg-dim)] hover:text-[var(--ui-fg)] px-1"
        title="Next (Enter)"
      >
        v
      </button>
      <button
        onClick={onClose}
        className="text-[var(--ui-fg-dim)] hover:text-[var(--term-red)] px-1 ml-1"
        title="Close (Escape)"
      >
        x
      </button>
    </div>
  );
}
