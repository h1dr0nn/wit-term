import { useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function Header() {
  const appWindow = getCurrentWindow();

  const handleMinimize = useCallback(() => {
    appWindow.minimize();
  }, [appWindow]);

  const handleMaximize = useCallback(async () => {
    const isMaximized = await appWindow.isMaximized();
    if (isMaximized) {
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
      className="h-[38px] flex items-center shrink-0 select-none border-b border-[var(--ui-border)] z-50"
      style={{
        background: "rgba(var(--header-bg-rgb, 24, 24, 37), 0.88)",
        backdropFilter: "blur(20px)",
        WebkitBackdropFilter: "blur(20px)",
      }}
    >
      {/* macOS traffic lights spacer (handled natively by titleBarStyle: overlay) */}
      <div className="pl-[78px] macos-only" />

      {/* Title area */}
      <div
        data-tauri-drag-region
        className="flex-1 flex items-center justify-center text-xs text-[var(--ui-fg-dim)] truncate"
      >
        Wit
      </div>

      {/* Window controls (Windows/Linux) */}
      <div className="flex items-center h-full non-macos-only">
        <button
          onClick={handleMinimize}
          className="w-[46px] h-full flex items-center justify-center text-[var(--ui-fg-dim)] hover:bg-[var(--ui-bg-tertiary)] transition-colors"
          title="Minimize"
        >
          <svg width="10" height="1" viewBox="0 0 10 1" fill="currentColor">
            <rect width="10" height="1" />
          </svg>
        </button>
        <button
          onClick={handleMaximize}
          className="w-[46px] h-full flex items-center justify-center text-[var(--ui-fg-dim)] hover:bg-[var(--ui-bg-tertiary)] transition-colors"
          title="Maximize"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1">
            <rect x="0.5" y="0.5" width="9" height="9" />
          </svg>
        </button>
        <button
          onClick={handleClose}
          className="w-[46px] h-full flex items-center justify-center text-[var(--ui-fg-dim)] hover:bg-[var(--term-red)] hover:text-white transition-colors"
          title="Close"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.2">
            <line x1="1" y1="1" x2="9" y2="9" />
            <line x1="9" y1="1" x2="1" y2="9" />
          </svg>
        </button>
      </div>
    </header>
  );
}
