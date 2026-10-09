export type StatusFilter = "all" | "dream" | "been";

interface Props {
  value: StatusFilter;
  onChange: (value: StatusFilter) => void;
  doneLabel?: string;
}

export default function FilterBar({ value, onChange, doneLabel }: Props) {
  return (
    <div className="filter-bar" role="group" aria-label="Filter">
      {(["all", "dream", "been"] as const).map((key) => (
        <button
          key={key}
          type="button"
          className={`text${value === key ? " current" : ""}`}
          onClick={() => onChange(key)}
        >
          {key === "all" ? "All" : key === "dream" ? "Dream" : (doneLabel ?? "Been")}
        </button>
      ))}
    </div>
  );
}
