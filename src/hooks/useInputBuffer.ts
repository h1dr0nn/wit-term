import { useRef, useCallback } from "react";

/**
 * Tracks the current command-line input buffer.
 *
 * Since we can't read back from the PTY what's on the current line,
 * we intercept keystrokes sent to the terminal and maintain a shadow
 * buffer. Resets on Enter (command submitted) or prompt detection.
 */
export function useInputBuffer() {
  const bufferRef = useRef("");
  const cursorRef = useRef(0);

  const append = useCallback((data: string) => {
    // Handle special sequences
    if (data === "\r" || data === "\n") {
      // Command submitted — reset
      bufferRef.current = "";
      cursorRef.current = 0;
      return;
    }

    if (data === "\x7f") {
      // Backspace
      if (cursorRef.current > 0) {
        const buf = bufferRef.current;
        bufferRef.current =
          buf.slice(0, cursorRef.current - 1) + buf.slice(cursorRef.current);
        cursorRef.current--;
      }
      return;
    }

    if (data === "\x1b[3~") {
      // Delete key
      const buf = bufferRef.current;
      if (cursorRef.current < buf.length) {
        bufferRef.current =
          buf.slice(0, cursorRef.current) + buf.slice(cursorRef.current + 1);
      }
      return;
    }

    // Arrow left
    if (data === "\x1b[D") {
      if (cursorRef.current > 0) cursorRef.current--;
      return;
    }

    // Arrow right
    if (data === "\x1b[C") {
      if (cursorRef.current < bufferRef.current.length) cursorRef.current++;
      return;
    }

    // Home
    if (data === "\x1b[H") {
      cursorRef.current = 0;
      return;
    }

    // End
    if (data === "\x1b[F") {
      cursorRef.current = bufferRef.current.length;
      return;
    }

    // Ctrl+U (kill line)
    if (data === "\x15") {
      bufferRef.current = bufferRef.current.slice(cursorRef.current);
      cursorRef.current = 0;
      return;
    }

    // Ctrl+K (kill to end)
    if (data === "\x0b") {
      bufferRef.current = bufferRef.current.slice(0, cursorRef.current);
      return;
    }

    // Ctrl+W (delete word backward)
    if (data === "\x17") {
      const buf = bufferRef.current;
      let pos = cursorRef.current;
      // Skip trailing spaces
      while (pos > 0 && buf[pos - 1] === " ") pos--;
      // Delete word
      while (pos > 0 && buf[pos - 1] !== " ") pos--;
      bufferRef.current = buf.slice(0, pos) + buf.slice(cursorRef.current);
      cursorRef.current = pos;
      return;
    }

    // Ctrl+C — reset
    if (data === "\x03") {
      bufferRef.current = "";
      cursorRef.current = 0;
      return;
    }

    // Ignore other escape sequences and control chars
    if (data.startsWith("\x1b") || (data.length === 1 && data.charCodeAt(0) < 32)) {
      return;
    }

    // Regular text — insert at cursor
    const buf = bufferRef.current;
    bufferRef.current =
      buf.slice(0, cursorRef.current) + data + buf.slice(cursorRef.current);
    cursorRef.current += data.length;
  }, []);

  const reset = useCallback(() => {
    bufferRef.current = "";
    cursorRef.current = 0;
  }, []);

  const getBuffer = useCallback(() => bufferRef.current, []);
  const getCursor = useCallback(() => cursorRef.current, []);

  /** Insert completion text, replacing the current word */
  const insertCompletion = useCallback((completionText: string): string => {
    const buf = bufferRef.current;
    const cursor = cursorRef.current;

    // Find the start of the current word
    let wordStart = cursor;
    while (wordStart > 0 && buf[wordStart - 1] !== " ") {
      wordStart--;
    }

    const currentWord = buf.slice(wordStart, cursor);

    // Calculate what to send to the terminal:
    // Delete the current word chars, then type the completion
    // Using backspaces to delete, then the new text
    const backspaces = "\x7f".repeat(currentWord.length);
    const newInput = backspaces + completionText;

    // Update our buffer
    bufferRef.current =
      buf.slice(0, wordStart) + completionText + buf.slice(cursor);
    cursorRef.current = wordStart + completionText.length;

    return newInput;
  }, []);

  return { append, reset, getBuffer, getCursor, insertCompletion };
}
