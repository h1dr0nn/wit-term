import { useEffect, useRef, useState, useCallback } from "react";
import { TerminalView } from "./components/terminal/TerminalView";
import { TabBar } from "./components/tabs/TabBar";
import { SessionSidebar } from "./components/sidebar/SessionSidebar";
import { ContextSidebar } from "./components/sidebar/ContextSidebar";
import { SettingsModal } from "./components/settings/SettingsModal";
import { CommandPalette } from "./components/CommandPalette";
import { useSessionStore } from "./stores/sessionStore";
import { useTheme } from "./hooks/useTheme";

function App() {
  const [sidebarVisible, setSidebarVisible] = useState(false);
  const [contextSidebarVisible, setContextSidebarVisible] = useState(false);
  const [settingsVisible, setSettingsVisible] = useState(false);
  const [paletteVisible, setPaletteVisible] = useState(false);
  const sessions = useSessionStore((s) => s.sessions);
  const createNewSession = useSessionStore((s) => s.createNewSession);

  // Initialize theme system
  useTheme();

  // Create first session on mount (guard against StrictMode double-invoke)
  const initRef = useRef(false);
  useEffect(() => {
    if (!initRef.current) {
      initRef.current = true;
      createNewSession();
    }
  }, [createNewSession]);

  const toggleSidebar = useCallback(() => setSidebarVisible((v) => !v), []);
  const toggleContextSidebar = useCallback(
    () => setContextSidebarVisible((v) => !v),
    [],
  );
  const openSettings = useCallback(() => setSettingsVisible(true), []);

  // Global keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      // Ctrl+B = toggle session sidebar
      if (e.ctrlKey && !e.shiftKey && (e.key === "b" || e.key === "B")) {
        e.preventDefault();
        toggleSidebar();
      }
      // Ctrl+Shift+B = toggle context sidebar
      if (e.ctrlKey && e.shiftKey && (e.key === "b" || e.key === "B")) {
        e.preventDefault();
        toggleContextSidebar();
      }
      // Ctrl+, = settings
      if (e.ctrlKey && e.key === ",") {
        e.preventDefault();
        setSettingsVisible((v) => !v);
      }
      // Ctrl+Shift+P = command palette
      if (e.ctrlKey && e.shiftKey && (e.key === "p" || e.key === "P")) {
        e.preventDefault();
        setPaletteVisible((v) => !v);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [toggleSidebar, toggleContextSidebar]);

  return (
    <div className="flex h-screen w-screen bg-[var(--ui-bg)]">
      {sidebarVisible && <SessionSidebar />}
      <div className="flex-1 flex flex-col min-w-0">
        {sessions.length > 1 && <TabBar />}
        <main className="flex-1 flex min-h-0">
          <div className="flex-1 flex flex-col min-w-0">
            <TerminalView />
          </div>
          {contextSidebarVisible && <ContextSidebar />}
        </main>
      </div>
      <SettingsModal
        visible={settingsVisible}
        onClose={() => setSettingsVisible(false)}
      />
      <CommandPalette
        visible={paletteVisible}
        onClose={() => setPaletteVisible(false)}
        onOpenSettings={openSettings}
        onToggleSidebar={toggleSidebar}
        onToggleContextSidebar={toggleContextSidebar}
      />
    </div>
  );
}

export default App;
