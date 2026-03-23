import { useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
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
      style={{
        height: "var(--header-height)",
        background: "rgba(13, 17, 23, 0.88)",
        backdropFilter: "blur(20px)",
        WebkitBackdropFilter: "blur(20px)",
        borderBottom: "1px solid rgba(48, 54, 61, 0.6)",
        zIndex: 100,
      }}
      className="flex items-center shrink-0 select-none"
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
          <svg width="10" height="1" viewBox="0 0 10 1" fill="currentColor"><rect width="10" height="1" /></svg>
        </button>
        <button
          onClick={handleMaximize}
          className="flex items-center justify-center transition-colors"
          style={{ width: 46, height: "100%", color: "var(--color-text-secondary)" }}
          onMouseEnter={(e) => { e.currentTarget.style.backgroundColor = "var(--color-surface-hover)"; }}
          onMouseLeave={(e) => { e.currentTarget.style.backgroundColor = "transparent"; }}
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1"><rect x="0.5" y="0.5" width="9" height="9" /></svg>
        </button>
        <button
          onClick={handleClose}
          className="flex items-center justify-center transition-colors"
          style={{ width: 46, height: "100%", color: "var(--color-text-secondary)" }}
          onMouseEnter={(e) => { e.currentTarget.style.backgroundColor = "#C42B1C"; e.currentTarget.style.color = "white"; }}
          onMouseLeave={(e) => { e.currentTarget.style.backgroundColor = "transparent"; e.currentTarget.style.color = "var(--color-text-secondary)"; }}
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.2"><line x1="1" y1="1" x2="9" y2="9" /><line x1="9" y1="1" x2="1" y2="9" /></svg>
        </button>
      </div>
    </header>
  );
}
