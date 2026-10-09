import { useEffect, useRef, useState, type KeyboardEvent } from "react";

interface Props {
  value: string;
  onSave: (next: string) => void | Promise<void>;
  className?: string;
  as?: "h1" | "span";
}

export default function InlineText({
  value,
  onSave,
  className = "",
  as = "span",
}: Props) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setDraft(value);
  }, [value]);

  useEffect(() => {
    if (editing) inputRef.current?.select();
  }, [editing]);

  async function commit() {
    const next = draft.trim();
    setEditing(false);
    if (!next || next === value) {
      setDraft(value);
      return;
    }
    await onSave(next);
  }

  function onKey(e: KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter") {
      e.preventDefault();
      void commit();
    } else if (e.key === "Escape") {
      setDraft(value);
      setEditing(false);
    }
  }

  if (editing) {
    return (
      <input
        ref={inputRef}
        className={className}
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={() => void commit()}
        onKeyDown={onKey}
        aria-label="Rename"
      />
    );
  }

  const Tag = as;
  return (
    <Tag
      className={`inline-text ${className}`.trim()}
      onClick={() => setEditing(true)}
      title="Click to rename"
    >
      {value}
    </Tag>
  );
}
