import { useEffect, useRef } from "react";
import type { AgentTimelineEvent } from "../../stores/agentStore";

function formatRelativeTime(timestamp: number): string {
  const now = Date.now();
  const diff = Math.max(0, Math.floor((now - timestamp) / 1000));
  if (diff < 5) return "just now";
  if (diff < 60) return `${diff}s ago`;
  const mins = Math.floor(diff / 60);
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  return `${hours}h ago`;
}

function eventDotClass(eventType: string): string {
  switch (eventType) {
    case "thinking_start":
    case "thinking_end":
      return "bg-blue-400";
    case "tool_use":
      return "bg-yellow-400";
    case "file_edit":
      return "bg-green-400";
    case "error":
      return "bg-red-400";
    default:
      return "bg-gray-400";
  }
}

function eventDescription(event: AgentTimelineEvent): { primary: string; secondary?: string } {
  const data = event.data;
  switch (event.eventType) {
    case "thinking_start":
      return { primary: "Thinking..." };
    case "thinking_end":
      return { primary: "Done thinking" };
    case "tool_use":
      return {
        primary: `Tool: ${String(data.tool_name ?? "unknown")}`,
        secondary: data.description ? String(data.description) : undefined,
      };
    case "file_edit":
      return { primary: `${String(data.action ?? "edited")} ${String(data.path ?? "")}` };
    case "token_update":
      return { primary: `Tokens: ${String(data.input ?? 0)} in / ${String(data.output ?? 0)} out` };
    case "cost_update":
      return { primary: `Cost: $${Number(data.total_cost ?? 0).toFixed(2)}` };
    case "model_info":
      return { primary: `Model: ${String(data.model_name ?? "unknown")}` };
    case "error":
      return { primary: String(data.message ?? data.error ?? "Error") };
    case "status_text":
      return { primary: String(data.text ?? "") };
    default:
      return { primary: event.eventType };
  }
}

interface AgentTimelineProps {
  events: AgentTimelineEvent[];
  isThinking: boolean;
}

export function AgentTimeline({ events, isThinking }: AgentTimelineProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [events.length, isThinking]);

  if (events.length === 0 && !isThinking) {
    return (
      <div className="py-20 px-4 text-center text-xs text-[var(--color-text-muted)]">
        No activity yet
      </div>
    );
  }

  return (
    <div ref={scrollRef} className="flex-1 overflow-y-auto custom-scrollbar px-3 py-2">
      {events.map((event) => {
        const desc = eventDescription(event);
        return (
          <div key={event.id} className="flex items-start gap-2 py-1.5">
            {/* Dot */}
            <div className="flex-shrink-0 mt-1.5">
              <div
                className={`w-2 h-2 rounded-full ${eventDotClass(event.eventType)}`}
              />
            </div>
            {/* Content */}
            <div className="flex-1 min-w-0">
              <span
                className={`text-xs leading-tight ${event.eventType === "error" ? "text-red-400" : "text-[var(--color-text-secondary)]"}`}
              >
                {desc.primary}
              </span>
              {desc.secondary && (
                <div className="text-xs text-[var(--color-text-muted)] truncate mt-0.5">
                  {desc.secondary}
                </div>
              )}
            </div>
            {/* Timestamp */}
            <span className="flex-shrink-0 text-[10px] text-[var(--color-text-muted)] mt-0.5">
              {formatRelativeTime(event.timestamp)}
            </span>
          </div>
        );
      })}

      {/* Thinking indicator */}
      {isThinking && (
        <div className="flex items-start gap-2 py-1.5">
          <div className="flex-shrink-0 mt-1.5">
            <div className="w-2 h-2 rounded-full bg-blue-400 animate-pulse" />
          </div>
          <span className="text-xs text-blue-400">Thinking...</span>
        </div>
      )}
    </div>
  );
}
