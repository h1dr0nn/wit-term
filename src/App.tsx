import { useEffect, useState } from "react";
import { TerminalView } from "./components/terminal/TerminalView";
import { TabBar } from "./components/tabs/TabBar";
import { SessionSidebar } from "./components/sidebar/SessionSidebar";
import { SettingsModal } from "./components/settings/SettingsModal";
import { useSessionStore } from "./stores/sessionStore";
import { useTheme } from "./hooks/useTheme";

function App() {
  const [sidebarVisible, setSidebarVisible] = useState(false);
  const [settingsVisible, setSettingsVisible] = useState(false);
  const sessions = useSessionStore((s) => s.sessions);
  const createNewSession = useSessionStore((s) => s.createNewSession);

  // Initialize theme system
  useTheme();

  // Create first session on mount
  useEffect(() => {
    if (sessions.length === 0) {
      createNewSession();
    }
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // Global keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      // Ctrl+B = toggle sidebar
      if (e.ctrlKey && !e.shiftKey && (e.key === "b" || e.key === "B")) {
        e.preventDefault();
        setSidebarVisible((v) => !v);
      }
      // Ctrl+, = settings
      if (e.ctrlKey && e.key === ",") {
        e.preventDefault();
        setSettingsVisible((v) => !v);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  return (
    <div className="flex h-screen w-screen bg-[var(--ui-bg)]">
      {sidebarVisible && <SessionSidebar />}
      <div className="flex-1 flex flex-col min-w-0">
        {sessions.length > 1 && <TabBar />}
        <main className="flex-1 flex flex-col min-h-0">
          <TerminalView />
        </main>
      </div>
      <SettingsModal
        visible={settingsVisible}
        onClose={() => setSettingsVisible(false)}
      />
    </div>
  );
}

export default App;
