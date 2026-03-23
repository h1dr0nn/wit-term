import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { X } from "lucide-react";

interface Notification {
  id: number;
  message: string;
  type: "info" | "warning" | "error" | "bell";
  timestamp: number;
}

let nextId = 0;

export function Notifications() {
  const [notifications, setNotifications] = useState<Notification[]>([]);

  const addNotification = useCallback(
    (message: string, type: Notification["type"] = "info") => {
      const id = nextId++;
      setNotifications((prev) => [...prev, { id, message, type, timestamp: Date.now() }]);

      // Auto-dismiss after 4 seconds
      setTimeout(() => {
        setNotifications((prev) => prev.filter((n) => n.id !== id));
      }, 4000);
    },
    [],
  );

  const dismiss = useCallback((id: number) => {
    setNotifications((prev) => prev.filter((n) => n.id !== id));
  }, []);

  useEffect(() => {
    const unlisteners: (() => void)[] = [];

    listen<{ session_id: string }>("terminal_bell", () => {
      addNotification("Bell", "bell");
    }).then((u) => unlisteners.push(u));

    return () => {
      unlisteners.forEach((u) => u());
    };
  }, [addNotification]);

  if (notifications.length === 0) return null;

  return (
    <div
      style={{
        position: "fixed",
        top: 56,
        right: 16,
        zIndex: 9999,
        display: "flex",
        flexDirection: "column",
        gap: 8,
        pointerEvents: "none",
      }}
    >
      {notifications.map((n) => (
        <div
          key={n.id}
          style={{
            background: "var(--color-surface)",
            border: `1px solid ${
              n.type === "error"
                ? "var(--color-error)"
                : n.type === "warning"
                  ? "var(--color-warning)"
                  : "var(--color-border)"
            }`,
            borderRadius: "var(--radius-md)",
            padding: "8px 12px",
            fontSize: 12,
            color: "var(--color-text)",
            boxShadow: "0 4px 12px rgba(0,0,0,0.3)",
            display: "flex",
            alignItems: "center",
            gap: 8,
            pointerEvents: "auto",
            minWidth: 200,
            maxWidth: 320,
          }}
        >
          <span style={{ flex: 1 }}>{n.message}</span>
          <button
            onClick={() => dismiss(n.id)}
            style={{
              background: "none",
              border: "none",
              color: "var(--color-text-muted)",
              cursor: "pointer",
              padding: 2,
              lineHeight: 1,
            }}
          >
            <X size={12} />
          </button>
        </div>
      ))}
    </div>
  );
}
