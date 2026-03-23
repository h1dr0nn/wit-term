import { useCompletionStore } from "../../stores/completionStore";

interface InlineHintProps {
  style?: React.CSSProperties;
}

export function InlineHint({ style }: InlineHintProps) {
  const inlineHint = useCompletionStore((s) => s.inlineHint);

  if (!inlineHint) return null;

  return (
    <span
      style={{
        color: "var(--color-text-muted)",
        opacity: 0.5,
        fontFamily: "var(--font-mono)",
        pointerEvents: "none",
        userSelect: "none",
        ...style,
      }}
    >
      {inlineHint}
    </span>
  );
}
