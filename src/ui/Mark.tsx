interface Props {
  been: boolean;
  onToggle: () => void;
  title?: string;
  onLabel?: string;
}

export default function Mark({ been, onToggle, title, onLabel }: Props) {
  const onName = onLabel ?? "been";
  return (
    <button
      type="button"
      className={`mark${been ? " been" : ""}`}
      title={title ?? (been ? onName[0].toUpperCase() + onName.slice(1) : "Dream")}
      aria-label={been ? "Mark as dream" : `Mark as ${onName}`}
      onClick={(e) => {
        e.stopPropagation();
        onToggle();
      }}
    />
  );
}
