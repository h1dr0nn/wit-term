import { useTerminalStore } from "../stores/terminalStore";
import { useSessionStore } from "../stores/sessionStore";

export function useTerminal() {
  const activeSessionId = useSessionStore((s) => s.activeSessionId);
  const grids = useTerminalStore((s) => s.grids);

  const snapshot = activeSessionId ? grids.get(activeSessionId) : undefined;

  return {
    activeSessionId,
    snapshot,
  };
}
