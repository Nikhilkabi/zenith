import {
  useEffect,
  useRef,
  useState,
  type DragEvent,
  type KeyboardEvent,
  type ReactNode,
} from "react";
import { assetUrl } from "../lib/assets";
import { primaryFor } from "../types";
import CoverImage from "./CoverImage";
import Mark from "./Mark";

interface Props {
  id: string;
  name: string;
  coverRelpath: string | null;
  coverX?: number;
  coverY?: number;
  libraryRoot: string;
  countLabel?: string;
  been?: boolean;
  markLabel?: string;
  onOpen: () => void;
  onToggleBeen?: () => void;
  onDropFiles?: (files: FileList) => void;
  onReorder?: (fromId: string, toId: string) => void;
}

export function Tile({
  id,
  name,
  coverRelpath,
  coverX = 50,
  coverY = 50,
  libraryRoot,
  countLabel,
  been,
  markLabel,
  onOpen,
  onToggleBeen,
  onDropFiles,
  onReorder,
}: Props) {
  const [drop, setDrop] = useState(false);
  const dragged = useRef(false);
  const bg = primaryFor(id);

  return (
    <button
      type="button"
      className={`tile${drop ? " drop-target" : ""}`}
      draggable={Boolean(onReorder)}
      onClick={() => {
        if (dragged.current) {
          dragged.current = false;
          return;
        }
        onOpen();
      }}
      onDragStart={(e: DragEvent) => {
        if (!onReorder) return;
        dragged.current = true;
        e.dataTransfer.setData("text/tile-id", id);
        e.dataTransfer.effectAllowed = "move";
      }}
      onDragOver={(e: DragEvent) => {
        if (onDropFiles || onReorder) {
          e.preventDefault();
          setDrop(true);
        }
      }}
      onDragLeave={() => setDrop(false)}
      onDrop={(e: DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        setDrop(false);
        if (e.dataTransfer.files.length && onDropFiles) {
          onDropFiles(e.dataTransfer.files);
          return;
        }
        const fromId = e.dataTransfer.getData("text/tile-id");
        if (fromId && onReorder && fromId !== id) onReorder(fromId, id);
      }}
    >
      <span className="tile-cover">
        {coverRelpath ? (
          <CoverImage
            src={assetUrl(libraryRoot, coverRelpath)}
            focusX={coverX}
            focusY={coverY}
          />
        ) : (
          <span className="tile-fallback" style={{ background: bg }} />
        )}
      </span>
      <span className="tile-body">
        <span className="tile-name">{name}</span>
        <span className="tile-side">
          {countLabel != null && <span className="tile-count">{countLabel}</span>}
          {onToggleBeen && (
            <Mark been={been === true} onLabel={markLabel} onToggle={onToggleBeen} />
          )}
        </span>
      </span>
    </button>
  );
}

interface Suggestion {
  label: string;
  iso?: string;
}

interface AddProps {
  placeholder: string;
  onCreate: (name: string, iso?: string) => void | Promise<void>;
  suggestionsFor?: (query: string) => Suggestion[];
}

export function AddTile({ placeholder, onCreate, suggestionsFor }: AddProps) {
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState("");
  const [active, setActive] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const suggestions = suggestionsFor ? suggestionsFor(name) : [];

  useEffect(() => {
    if (editing) inputRef.current?.focus();
  }, [editing]);

  async function submit(chosen?: Suggestion) {
    const trimmed = (chosen?.label ?? name).trim();
    if (!trimmed) {
      setEditing(false);
      setName("");
      return;
    }
    setName("");
    setEditing(false);
    await onCreate(trimmed, chosen?.iso);
  }

  function onKey(e: KeyboardEvent<HTMLInputElement>) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActive((i) => Math.min(i + 1, Math.max(suggestions.length - 1, 0)));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActive((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      void submit(suggestions[active]);
    } else if (e.key === "Escape") {
      setName("");
      setEditing(false);
    }
  }

  if (editing) {
    return (
      <div className="tile add editing">
        <input
          ref={inputRef}
          value={name}
          placeholder={placeholder}
          onChange={(e) => {
            setName(e.target.value);
            setActive(0);
          }}
          onBlur={() => {
            if (!name.trim()) {
              setEditing(false);
              return;
            }
            void submit(suggestions[active]);
          }}
          onKeyDown={onKey}
        />
        {suggestions.length > 0 && (
          <ul className="add-suggest">
            {suggestions.map((s, i) => (
              <li key={s.label}>
                <button
                  type="button"
                  className={i === active ? "active" : undefined}
                  onMouseDown={(e) => {
                    e.preventDefault();
                    void submit(s);
                  }}
                >
                  {s.label}
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    );
  }

  return (
    <button type="button" className="tile add" onClick={() => setEditing(true)} aria-label="Add">
      +
    </button>
  );
}

export function TileGrid({ children }: { children: ReactNode }) {
  return <div className="tile-grid">{children}</div>;
}
