import { useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";

export function Header() {
  const appWindow = getCurrentWindow();

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

  return (
    <header
      data-tauri-drag-region
      className="flex items-center shrink-0 select-none h-10 z-[100] border-b border-[var(--color-border-muted)] bg-[var(--color-bg)]/80 backdrop-blur-2xl"
    >
      {/* macOS traffic lights spacer */}
      <div className="macos-only" style={{ width: 78 }} />

      {/* Drag region fills center */}
      <div data-tauri-drag-region className="flex-1" />

      {/* Window controls (Windows/Linux only) */}
      <div className="flex items-center non-macos-only" style={{ height: "100%" }}>
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
