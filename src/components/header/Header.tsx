import { useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";
import { useSessionStore } from "../../stores/sessionStore";


export function Header() {
  const appWindow = getCurrentWindow();
  const activeSessionId = useSessionStore((s) => s.activeSessionId);
  const sessions = useSessionStore((s) => s.sessions);
  const activeSession = sessions.find((s) => s.id === activeSessionId);

  const handleMinimize = useCallback(() => {
    appWindow.minimize();
  }, [appWindow]);

  const handleMaximize = useCallback(async () => {
    if (await appWindow.isMaximized()) {
      appWindow.unmaximize();
    } else {
      appWindow.maximize();
    }
  }, [appWindow]);

  const handleClose = useCallback(() => {
    appWindow.close();
  }, [appWindow]);

  const title = activeSession?.title || "Wit";

  return (
    <header
      data-tauri-drag-region
      className="flex items-center shrink-0 select-none h-10 z-[100] border-b border-[var(--color-border-muted)] bg-[var(--color-bg)]/80 backdrop-blur-2xl"
    >

      {/* macOS traffic lights spacer */}
      <div className="macos-only" style={{ width: 78 }} />

      {/* Left padding (Windows/Linux) */}
      <div className="non-macos-only" style={{ width: 12 }} />

      {/* Title area - centered */}
      <div
        data-tauri-drag-region
        className="flex-1 flex items-center justify-center min-w-0"
      >
        <span
          data-tauri-drag-region
          style={{
            fontSize: 13,
            fontWeight: 500,
            color: "var(--color-text-secondary)",
          }}
          className="truncate"
        >
          {title}
        </span>
      </div>

      {/* Window controls (Windows/Linux only) */}
      <div className="flex items-center non-macos-only" style={{ height: "var(--header-height)" }}>
        <button
          onClick={handleMinimize}
          className="flex items-center justify-center transition-colors"
          style={{ width: 46, height: "100%", color: "var(--color-text-secondary)" }}
          onMouseEnter={(e) => { e.currentTarget.style.backgroundColor = "var(--color-surface-hover)"; }}
          onMouseLeave={(e) => { e.currentTarget.style.backgroundColor = "transparent"; }}
        >
          <Minus size={14} strokeWidth={2} />
        </button>
        <button
          onClick={handleMaximize}
          className="flex items-center justify-center transition-colors"
          style={{ width: 46, height: "100%", color: "var(--color-text-secondary)" }}
          onMouseEnter={(e) => { e.currentTarget.style.backgroundColor = "var(--color-surface-hover)"; }}
          onMouseLeave={(e) => { e.currentTarget.style.backgroundColor = "transparent"; }}
        >
          <Square size={12} strokeWidth={2} />
        </button>
        <button
          onClick={handleClose}
          className="flex items-center justify-center transition-colors"
          style={{ width: 46, height: "100%", color: "var(--color-text-secondary)" }}
          onMouseEnter={(e) => { e.currentTarget.style.backgroundColor = "#C42B1C"; e.currentTarget.style.color = "white"; }}
          onMouseLeave={(e) => { e.currentTarget.style.backgroundColor = "transparent"; e.currentTarget.style.color = "var(--color-text-secondary)"; }}
        >
          <X size={14} strokeWidth={2} />
        </button>
      </div>
    </header>
  );
}
