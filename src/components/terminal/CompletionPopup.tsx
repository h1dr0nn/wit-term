import React from "react";
import { useCompletionStore, type CompletionItem } from "../../stores/completionStore";

const KIND_ICONS: Record<string, string> = {
  Command: "CMD",
  Subcommand: "SUB",
  Flag: "FLG",
  Argument: "ARG",
  Path: "PTH",
};

const KIND_COLORS: Record<string, string> = {
  Command: "text-[#89b4fa]",
  Subcommand: "text-[#a6e3a1]",
  Flag: "text-[#f9e2af]",
  Argument: "text-[#f5c2e7]",
  Path: "text-[#94e2d5]",
};

export function CompletionPopup() {
  const visible = useCompletionStore((s) => s.visible);
  const items = useCompletionStore((s) => s.items);
  const selectedIndex = useCompletionStore((s) => s.selectedIndex);

  if (!visible || items.length === 0) return null;

  return (
    <div className="absolute bottom-8 left-1 z-50 max-h-64 min-w-64 max-w-96 overflow-auto rounded border border-[var(--ui-border)] bg-[var(--ui-bg-tertiary)] shadow-lg font-mono text-sm">
      {items.map((item, idx) => (
        <CompletionRow key={idx} item={item} selected={idx === selectedIndex} />
      ))}
    </div>
  );
}

interface CompletionRowProps {
  item: CompletionItem;
  selected: boolean;
}

const CompletionRow = React.memo(function CompletionRow({ item, selected }: CompletionRowProps) {
  const kindLabel = KIND_ICONS[item.kind] || item.kind;
  const kindColor = KIND_COLORS[item.kind] || "text-[#a6adc8]";

  return (
    <div
      className={`flex items-center gap-2 px-2 py-1 ${
        selected ? "bg-[var(--ui-border)]" : "hover:bg-[var(--ui-bg-secondary)]"
      }`}
    >
      <span className={`text-xs font-bold w-8 shrink-0 ${kindColor}`}>{kindLabel}</span>
      <span className="text-[var(--ui-fg)] truncate flex-1">{item.display}</span>
      {item.description && (
        <span className="text-[var(--ui-fg-dim)] text-xs truncate max-w-40">{item.description}</span>
      )}
    </div>
  );
});
