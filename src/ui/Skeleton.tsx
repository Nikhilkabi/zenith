interface Props {
  count?: number;
}

export default function Skeleton({ count = 4 }: Props) {
  return (
    <div className="skeleton-grid" aria-hidden>
      {Array.from({ length: count }, (_, i) => (
        <div key={i} className="skeleton" />
      ))}
    </div>
  );
}
