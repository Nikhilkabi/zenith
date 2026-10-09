import { useEffect, useRef, useState } from "react";
import { search as searchApi } from "../api";
import type { SearchHit } from "../types";

interface Props {
  query: string;
  onQuery: (value: string) => void;
  onOpen: (hit: SearchHit) => void;
  onError: (message: string | null) => void;
  inputRef: React.RefObject<HTMLInputElement | null>;
}

export default function Search({
  query,
  onQuery,
  onOpen,
  onError,
  inputRef,
}: Props) {
  const [hits, setHits] = useState<SearchHit[]>([]);
  const [active, setActive] = useState(0);
  const wrapRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!query.trim()) {
      setHits([]);
      return;
    }
    const handle = window.setTimeout(() => {
      searchApi(query)
        .then((next) => {
          setHits(next);
          setActive(0);
        })
        .catch((err) => onError(String(err)));
    }, 150);
    return () => window.clearTimeout(handle);
  }, [query, onError]);

  function closeResults() {
    onQuery("");
    setHits([]);
  }

  return (
    <div className="search-box" ref={wrapRef}>
      <input
        ref={inputRef}
        value={query}
        placeholder="Search"
        onChange={(e) => onQuery(e.target.value)}
        onBlur={(e) => {
          const next = e.relatedTarget as Node | null;
          if (wrapRef.current && next && wrapRef.current.contains(next)) return;
          window.setTimeout(() => closeResults(), 100);
        }}
        onKeyDown={(e) => {
          if (e.key === "ArrowDown") {
            e.preventDefault();
            setActive((i) => Math.min(i + 1, Math.max(hits.length - 1, 0)));
          } else if (e.key === "ArrowUp") {
            e.preventDefault();
            setActive((i) => Math.max(i - 1, 0));
          } else if (e.key === "Enter" && hits[active]) {
            e.preventDefault();
            onOpen(hits[active]);
            closeResults();
          } else if (e.key === "Escape") {
            closeResults();
            inputRef.current?.blur();
          }
        }}
      />
      {hits.length > 0 && (
        <ul className="search-panel">
          {hits.map((hit, i) => (
            <li key={`${hit.kind}-${hit.id}`}>
              <button
                type="button"
                className={i === active ? "active" : undefined}
                onMouseEnter={() => setActive(i)}
                onClick={() => {
                  onOpen(hit);
                  onQuery("");
                  setHits([]);
                }}
              >
                <span className="kind">{hit.kind}</span>
                <span>{hit.title}</span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
