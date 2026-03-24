import { useEffect, useRef, useState, useCallback } from "react";
import { TerminalView } from "./components/terminal/TerminalView";
import { Header } from "./components/header/Header";
import { SessionSidebar } from "./components/sidebar/SessionSidebar";
import { ContextSidebar } from "./components/sidebar/ContextSidebar";
import { AgentSidebar } from "./components/sidebar/AgentSidebar";
import { SettingsModal } from "./components/settings/SettingsModal";
import { CommandPalette } from "./components/CommandPalette";
import { Notifications } from "./components/Notifications";
import { useSessionStore } from "./stores/sessionStore";
import { useAgentStore } from "./stores/agentStore";
import { useTheme } from "./hooks/useTheme";

function App() {
  const [sidebarVisible, setSidebarVisible] = useState(true);
  const [contextSidebarVisible, setContextSidebarVisible] = useState(false);
  const [settingsVisible, setSettingsVisible] = useState(false);
  const [paletteVisible, setPaletteVisible] = useState(false);
  const createNewSession = useSessionStore((s) => s.createNewSession);
  const activeSessionId = useSessionStore((s) => s.activeSessionId);
  const agentSidebarVisible = useAgentStore((s) => s.sidebarVisible);
  const toggleAgentSidebar = useAgentStore((s) => s.toggleSidebar);

  useTheme();

  const initRef = useRef(false);
  useEffect(() => {
    if (!initRef.current) {
      initRef.current = true;
      // Estimate initial terminal size from window dimensions
      // Subtract ~300px for sidebars/padding, use char metrics
      const availWidth = Math.max(400, window.innerWidth - 320);
      const availHeight = Math.max(200, window.innerHeight - 120);
      const cols = Math.max(1, Math.floor(availWidth / 8.4));
      const rows = Math.max(1, Math.floor(availHeight / (14 * 1.2)));
      createNewSession(undefined, cols, rows);
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
      if (e.ctrlKey && e.shiftKey && (e.key === "a" || e.key === "A")) {
        e.preventDefault();
        toggleAgentSidebar();
      }
      if (e.ctrlKey && e.shiftKey && (e.key === "p" || e.key === "P")) {
        e.preventDefault();
        setPaletteVisible((v) => !v);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [toggleSidebar, toggleContextSidebar, toggleAgentSidebar]);

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
          {agentSidebarVisible && activeSessionId && (
            <AgentSidebar sessionId={activeSessionId} />
          )}
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
      <Notifications />
    </div>
  );
}

export default App;
