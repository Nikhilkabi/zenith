interface Props {
  onClose: () => void;
}

const ROWS: [string, string][] = [
  ["/", "Search"],
  ["?", "This sheet"],
  ["← →", "Previous / next photo"],
  ["C", "Set place cover"],
  ["Shift+C", "Set country cover"],
  ["Delete", "Delete photo (twice)"],
  ["Esc", "Close"],
];

export default function CheatSheet({ onClose }: Props) {
  return (
    <div className="sheet" role="dialog" aria-modal aria-label="Keyboard">
      <div className="sheet-card">
        <div className="page-header">
          <h1 className="page-title">Keys</h1>
          <button type="button" className="text" onClick={onClose}>
            Close
          </button>
        </div>
        <dl className="sheet-list">
          {ROWS.map(([key, label]) => (
            <div key={key}>
              <dt>{key}</dt>
              <dd>{label}</dd>
            </div>
          ))}
        </dl>
      </div>
    </div>
  );
}
