import { useEffect, useRef, useState, useCallback } from "react";
import { TerminalView } from "./components/terminal/TerminalView";
import { Header } from "./components/header/Header";
import { SessionSidebar } from "./components/sidebar/SessionSidebar";
import { ContextSidebar } from "./components/sidebar/ContextSidebar";
import { SettingsModal } from "./components/settings/SettingsModal";
import { CommandPalette } from "./components/CommandPalette";
import { useSessionStore } from "./stores/sessionStore";
import { useTheme } from "./hooks/useTheme";

function App() {
  const [sidebarVisible, setSidebarVisible] = useState(true);
  const [contextSidebarVisible, setContextSidebarVisible] = useState(false);
  const [settingsVisible, setSettingsVisible] = useState(false);
  const [paletteVisible, setPaletteVisible] = useState(false);
  const createNewSession = useSessionStore((s) => s.createNewSession);

  useTheme();

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
  const openPalette = useCallback(() => setPaletteVisible(true), []);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.ctrlKey && !e.shiftKey && (e.key === "b" || e.key === "B")) {
        e.preventDefault();
        toggleSidebar();
      }
      if (e.ctrlKey && e.shiftKey && (e.key === "b" || e.key === "B")) {
        e.preventDefault();
        toggleContextSidebar();
      }
      if (e.ctrlKey && e.key === ",") {
        e.preventDefault();
        setSettingsVisible((v) => !v);
      }
      if (e.ctrlKey && e.shiftKey && (e.key === "p" || e.key === "P")) {
        e.preventDefault();
        setPaletteVisible((v) => !v);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [toggleSidebar, toggleContextSidebar]);

  return (
    <div className="window-root flex flex-col h-screen w-screen">
      <Header />
      <div className="flex flex-1 min-h-0">
        {sidebarVisible && (
          <SessionSidebar
            onOpenSettings={openSettings}
            onOpenPalette={openPalette}
          />
        )}
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
