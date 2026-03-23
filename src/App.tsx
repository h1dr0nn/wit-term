import { useEffect, useState } from "react";
import { TerminalView } from "./components/terminal/TerminalView";
import { TabBar } from "./components/tabs/TabBar";
import { SessionSidebar } from "./components/sidebar/SessionSidebar";
import { useSessionStore } from "./stores/sessionStore";

function App() {
  const [sidebarVisible, setSidebarVisible] = useState(false);
  const sessions = useSessionStore((s) => s.sessions);
  const createNewSession = useSessionStore((s) => s.createNewSession);

  // Create first session on mount
  useEffect(() => {
    if (sessions.length === 0) {
      createNewSession();
    }
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // Toggle sidebar with Ctrl+B
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.ctrlKey && !e.shiftKey && (e.key === "b" || e.key === "B")) {
        e.preventDefault();
        setSidebarVisible((v) => !v);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  return (
    <div className="flex h-screen w-screen bg-[#1e1e2e]">
      {sidebarVisible && <SessionSidebar />}
      <div className="flex-1 flex flex-col min-w-0">
        {sessions.length > 1 && <TabBar />}
        <main className="flex-1 flex flex-col min-h-0">
          <TerminalView />
        </main>
      </div>
    </div>
  );
}

export default App;
