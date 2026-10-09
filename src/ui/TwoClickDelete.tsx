import { useEffect, useRef, useState } from "react";

interface Props {
  label: string;
  confirmLabel?: string;
  onConfirm: () => void | Promise<void>;
  className?: string;
}

export default function TwoClickDelete({
  label,
  confirmLabel,
  onConfirm,
  className = "",
}: Props) {
  const [armed, setArmed] = useState(false);
  const ref = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!armed) return;
    function onDoc(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setArmed(false);
      }
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") setArmed(false);
    }
    document.addEventListener("mousedown", onDoc);
    window.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDoc);
      window.removeEventListener("keydown", onKey);
    };
  }, [armed]);

  return (
    <button
      ref={ref}
      type="button"
      className={`text ${armed ? "armed confirm" : "danger"} ${className}`.trim()}
      onClick={() => {
        if (!armed) {
          setArmed(true);
          return;
        }
        setArmed(false);
        void onConfirm();
      }}
    >
      {armed ? (confirmLabel ?? `Delete ${label}?`) : "Delete"}
    </button>
  );
}
