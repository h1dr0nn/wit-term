/**
 * Encode a keyboard event into terminal byte sequences.
 *
 * Returns null if the key should not be sent to the terminal
 * (e.g., it's a modifier key press alone).
 */
export function encodeKey(e: KeyboardEvent | React.KeyboardEvent): string | null {
  // Ignore standalone modifier keys
  if (
    e.key === "Shift" ||
    e.key === "Control" ||
    e.key === "Alt" ||
    e.key === "Meta" ||
    e.key === "CapsLock" ||
    e.key === "NumLock" ||
    e.key === "ScrollLock"
  ) {
    return null;
  }

  // Ctrl+Shift+C/V are clipboard operations, don't send to PTY
  if (e.ctrlKey && e.shiftKey && (e.key === "C" || e.key === "c")) return null;
  if (e.ctrlKey && e.shiftKey && (e.key === "V" || e.key === "v")) return null;

  // Ctrl+key -> control codes
  if (e.ctrlKey && !e.altKey && !e.metaKey && e.key.length === 1) {
    const code = e.key.toLowerCase().charCodeAt(0) - 96;
    if (code >= 1 && code <= 26) {
      return String.fromCharCode(code);
    }
    // Ctrl+[ = ESC
    if (e.key === "[") return "\x1b";
    // Ctrl+] = GS
    if (e.key === "]") return "\x1d";
    // Ctrl+\ = FS
    if (e.key === "\\") return "\x1c";
    // Ctrl+^ = RS
    if (e.key === "^" || e.key === "6") return "\x1e";
    // Ctrl+_ = US
    if (e.key === "_" || e.key === "-") return "\x1f";
    // Ctrl+Space = NUL
    if (e.key === " ") return "\x00";
  }

  // Alt+key -> ESC + key
  if (e.altKey && !e.ctrlKey && !e.metaKey && e.key.length === 1) {
    return "\x1b" + e.key;
  }

  // Special keys
  switch (e.key) {
    case "Enter":
      return "\r";
    case "Backspace":
      return "\x7f";
    case "Tab":
      return e.shiftKey ? "\x1b[Z" : "\t";
    case "Escape":
      return "\x1b";
    case "ArrowUp":
      return e.ctrlKey ? "\x1b[1;5A" : e.shiftKey ? "\x1b[1;2A" : "\x1b[A";
    case "ArrowDown":
      return e.ctrlKey ? "\x1b[1;5B" : e.shiftKey ? "\x1b[1;2B" : "\x1b[B";
    case "ArrowRight":
      return e.ctrlKey ? "\x1b[1;5C" : e.shiftKey ? "\x1b[1;2C" : "\x1b[C";
    case "ArrowLeft":
      return e.ctrlKey ? "\x1b[1;5D" : e.shiftKey ? "\x1b[1;2D" : "\x1b[D";
    case "Home":
      return "\x1b[H";
    case "End":
      return "\x1b[F";
    case "Insert":
      return "\x1b[2~";
    case "Delete":
      return "\x1b[3~";
    case "PageUp":
      return "\x1b[5~";
    case "PageDown":
      return "\x1b[6~";
    // Function keys
    case "F1":
      return "\x1bOP";
    case "F2":
      return "\x1bOQ";
    case "F3":
      return "\x1bOR";
    case "F4":
      return "\x1bOS";
    case "F5":
      return "\x1b[15~";
    case "F6":
      return "\x1b[17~";
    case "F7":
      return "\x1b[18~";
    case "F8":
      return "\x1b[19~";
    case "F9":
      return "\x1b[20~";
    case "F10":
      return "\x1b[21~";
    case "F11":
      return "\x1b[23~";
    case "F12":
      return "\x1b[24~";
  }

  // Regular printable character
  if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
    return e.key;
  }

  return null;
}
