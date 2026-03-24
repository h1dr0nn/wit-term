export interface UrlSpan {
  text: string;
  isUrl: boolean;
  url?: string;
}

/** Split text into segments, identifying URLs. */
export function detectUrls(text: string): UrlSpan[] {
  const urlRegex = /https?:\/\/[^\s<>"')\]]+/g;

  const spans: UrlSpan[] = [];
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = urlRegex.exec(text)) !== null) {
    // Add non-URL text before this match
    if (match.index > lastIndex) {
      spans.push({ text: text.slice(lastIndex, match.index), isUrl: false });
    }
    // Add the URL, trimming trailing punctuation
    const url = match[0].replace(/[.,;:!?)]+$/, "");
    spans.push({ text: url, isUrl: true, url });
    lastIndex = match.index + match[0].length;
    // If we trimmed punctuation, add it as non-URL text
    if (url.length < match[0].length) {
      const trailing = match[0].slice(url.length);
      spans.push({ text: trailing, isUrl: false });
    }
  }

  // Add remaining text
  if (lastIndex < text.length) {
    spans.push({ text: text.slice(lastIndex), isUrl: false });
  }

  return spans.length > 0 ? spans : [{ text, isUrl: false }];
}
