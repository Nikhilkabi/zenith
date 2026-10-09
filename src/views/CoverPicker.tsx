import { useCallback, useEffect, useRef, useState, type KeyboardEvent, type PointerEvent } from "react";
import {
  applyFileCover,
  applyStockCover,
  openUrl,
  searchCovers,
  setCoverFocus,
  type CoverTarget,
} from "../api";
import { assetUrl } from "../lib/assets";
import { focusAfterDrag, renderedCoverSize, roundFocus } from "../lib/coverFocus";
import { normalizeImageFile } from "../lib/normalizeImage";
import type { CoverHit } from "../types";
import CoverImage from "../ui/CoverImage";

interface CurrentCover {
  src: string;
  x: number;
  y: number;
}

interface Props {
  libraryRoot: string;
  initialQuery: string;
  target: CoverTarget;
  ownerId: string;
  shape: "wide" | "square";
  current: CurrentCover | null;
  startOnCurrent: boolean;
  onClose: () => void;
  onApplied: () => void | Promise<void>;
  onError: (message: string | null) => void;
}

type Draft =
  | { kind: "stock"; id: string; src: string; credit: string; x: number; y: number }
  | { kind: "file"; filename: string; bytes: number[]; src: string; x: number; y: number }
  | { kind: "current"; src: string; x: number; y: number };

export function CoverCredit({ credit, url }: { credit: string; url?: string | null }) {
  if (!url) return <span>{credit}</span>;
  return (
    <button type="button" className="text" onClick={() => void openUrl(url)}>
      {credit}
    </button>
  );
}

function CoverFrame({
  src,
  x,
  y,
  shape,
  onChange,
}: {
  src: string;
  x: number;
  y: number;
  shape: "wide" | "square";
  onChange: (x: number, y: number) => void;
}) {
  const frameRef = useRef<HTMLDivElement>(null);
  const drag = useRef<{ px: number; py: number; x: number; y: number } | null>(null);

  function shift(nextX: number, nextY: number, dx: number, dy: number) {
    const frame = frameRef.current;
    const img = frame?.querySelector("img");
    if (!frame || !img || img.naturalWidth < 1) return;
    const rendered = renderedCoverSize(
      img.naturalWidth,
      img.naturalHeight,
      frame.clientWidth,
      frame.clientHeight,
    );
    const next = focusAfterDrag(
      nextX,
      nextY,
      dx,
      dy,
      rendered.width,
      rendered.height,
      frame.clientWidth,
      frame.clientHeight,
    );
    onChange(next.x, next.y);
  }

  function onPointerDown(event: PointerEvent<HTMLDivElement>) {
    drag.current = { px: event.clientX, py: event.clientY, x, y };
    event.currentTarget.setPointerCapture(event.pointerId);
  }

  function onPointerMove(event: PointerEvent<HTMLDivElement>) {
    const start = drag.current;
    if (!start) return;
    const frame = frameRef.current;
    const img = frame?.querySelector("img");
    if (!frame || !img || img.naturalWidth < 1) return;
    const rendered = renderedCoverSize(
      img.naturalWidth,
      img.naturalHeight,
      frame.clientWidth,
      frame.clientHeight,
    );
    const next = focusAfterDrag(
      start.x,
      start.y,
      event.clientX - start.px,
      event.clientY - start.py,
      rendered.width,
      rendered.height,
      frame.clientWidth,
      frame.clientHeight,
    );
    onChange(next.x, next.y);
  }

  function onKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    const frame = frameRef.current;
    if (!frame) return;
    const step = (event.shiftKey ? 1 : 8) * (frame.clientWidth / 100);
    let dx = 0;
    let dy = 0;
    if (event.key === "ArrowLeft") dx = step;
    else if (event.key === "ArrowRight") dx = -step;
    else if (event.key === "ArrowUp") dy = step;
    else if (event.key === "ArrowDown") dy = -step;
    else return;
    event.preventDefault();
    shift(x, y, dx, dy);
  }

  return (
    <div
      ref={frameRef}
      className={`cover-frame${shape === "square" ? " square" : ""}`}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={() => {
        drag.current = null;
      }}
      onPointerCancel={() => {
        drag.current = null;
      }}
      onKeyDown={onKeyDown}
      tabIndex={0}
      role="application"
      aria-label="Drag the photo to choose what shows. Arrow keys nudge it."
    >
      <CoverImage src={src} focusX={x} focusY={y} loading="eager" />
    </div>
  );
}

export default function CoverPicker({
  libraryRoot,
  initialQuery,
  target,
  ownerId,
  shape,
  current,
  startOnCurrent,
  onClose,
  onApplied,
  onError,
}: Props) {
  const [query, setQuery] = useState(initialQuery);
  const [hits, setHits] = useState<CoverHit[]>([]);
  const [page, setPage] = useState(1);
  const [pageCount, setPageCount] = useState(1);
  const [searching, setSearching] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const [draft, setDraft] = useState<Draft | null>(() =>
    startOnCurrent && current
      ? { kind: "current", src: current.src, x: current.x, y: current.y }
      : null,
  );
  const fileRef = useRef<HTMLInputElement>(null);
  const fileUrl = useRef<string | null>(null);
  const gridRef = useRef<HTMLDivElement>(null);
  const sentinelRef = useRef<HTMLDivElement>(null);
  const request = useRef(0);
  const loadingMoreRef = useRef(false);

  useEffect(
    () => () => {
      if (fileUrl.current) URL.revokeObjectURL(fileUrl.current);
    },
    [],
  );

  const runSearch = useCallback(
    async (next: string) => {
      const trimmed = next.trim();
      if (!trimmed) return;
      const token = ++request.current;
      setSearching(true);
      setNote(null);
      setHits([]);
      try {
        const found = await searchCovers(trimmed, 1);
        if (token !== request.current) return;
        setHits(found.hits);
        setPage(found.page);
        setPageCount(found.page_count);
        if (found.warning) setNote(found.warning);
        else if (found.hits.length === 0) {
          setNote("Nothing for that. Try a simpler word, or use a photo of your own.");
        }
        onError(null);
      } catch (err) {
        if (token !== request.current) return;
        setHits([]);
        setNote(String(err));
      } finally {
        if (token === request.current) setSearching(false);
      }
    },
    [onError],
  );

  const loadMore = useCallback(async () => {
    if (loadingMoreRef.current || searching || page >= pageCount) return;
    const trimmed = query.trim();
    if (!trimmed) return;
    const token = request.current;
    loadingMoreRef.current = true;
    setLoadingMore(true);
    try {
      const found = await searchCovers(trimmed, page + 1);
      if (token !== request.current) return;
      setHits((prev) => {
        const seen = new Set(prev.map((hit) => hit.id));
        return [...prev, ...found.hits.filter((hit) => !seen.has(hit.id))];
      });
      setPage(found.page || page + 1);
      setPageCount(found.hits.length === 0 ? page : found.page_count);
      if (found.warning) setNote(found.warning);
    } catch (err) {
      if (token === request.current) setNote(String(err));
    } finally {
      loadingMoreRef.current = false;
      if (token === request.current) setLoadingMore(false);
    }
  }, [page, pageCount, query, searching]);

  useEffect(() => {
    if (startOnCurrent) return;
    void runSearch(initialQuery);
  }, [initialQuery, runSearch, startOnCurrent]);

  useEffect(() => {
    const root = gridRef.current;
    const node = sentinelRef.current;
    if (!root || !node || draft) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) void loadMore();
      },
      { root, rootMargin: "160px" },
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, [draft, hits.length, loadMore]);

  async function finish(work: () => Promise<void>) {
    try {
      await work();
      await onApplied();
      onClose();
    } catch (err) {
      onError(String(err));
    } finally {
      setBusyId(null);
    }
  }

  function clearFileUrl() {
    if (fileUrl.current) URL.revokeObjectURL(fileUrl.current);
    fileUrl.current = null;
  }

  function back() {
    if (draft?.kind === "file") clearFileUrl();
    setDraft(null);
    if (hits.length === 0) void runSearch(query);
  }

  function updateDraft(x: number, y: number) {
    setDraft((currentDraft) => (currentDraft ? { ...currentDraft, x, y } : currentDraft));
  }

  return (
    <div className="sheet" role="dialog" aria-modal aria-label="Choose a cover">
      <div className="sheet-card cover-sheet">
        <div className="page-header">
          <div>
            <h1 className="page-title">{draft ? "Position" : "Cover"}</h1>
            <p className="meta">
              {draft
                ? "Drag the photo so the part you want sits in the middle."
                : "Openverse, Pexels, and Pixabay. Add the Pexels and Pixabay keys from the menu. The photo you pick is saved in your library."}
            </p>
          </div>
          <button type="button" className="text" onClick={onClose}>
            Close
          </button>
        </div>

        {draft ? (
          <>
            <CoverFrame
              src={draft.src}
              x={draft.x}
              y={draft.y}
              shape={shape}
              onChange={updateDraft}
            />
            {draft.kind === "stock" && <p className="cover-credit">{draft.credit}</p>}
            <div className="cover-frame-actions">
              <button type="button" className="text" onClick={back} disabled={busyId != null}>
                Back
              </button>
              <button
                type="button"
                className="primary"
                disabled={busyId != null}
                onClick={() => {
                  const chosen = draft;
                  const x = roundFocus(chosen.x);
                  const y = roundFocus(chosen.y);
                  setBusyId("save");
                  void finish(async () => {
                    if (chosen.kind === "stock") {
                      await applyStockCover(target, ownerId, chosen.id, x, y);
                    } else if (chosen.kind === "file") {
                      await applyFileCover(target, ownerId, chosen.filename, chosen.bytes, x, y);
                    } else {
                      await setCoverFocus(target, ownerId, x, y);
                    }
                  });
                }}
              >
                {busyId === "save" ? "Saving" : "Use this cover"}
              </button>
            </div>
          </>
        ) : (
          <>
            <form
              className="cover-search"
              onSubmit={(event) => {
                event.preventDefault();
                void runSearch(query);
              }}
            >
              <input
                value={query}
                placeholder="Search photos"
                onChange={(event) => setQuery(event.target.value)}
              />
              <button type="submit" className="text" disabled={searching || busyId != null}>
                {searching ? "Searching" : "Search"}
              </button>
              <button
                type="button"
                className="text"
                disabled={busyId != null}
                onClick={() => fileRef.current?.click()}
              >
                Your photo
              </button>
              <input
                ref={fileRef}
                type="file"
                accept="image/jpeg,image/png,image/webp,image/avif,.jpg,.jpeg,.png,.webp,.avif"
                hidden
                onChange={(event) => {
                  const file = event.target.files?.[0];
                  event.target.value = "";
                  if (!file) return;
                  setBusyId("file");
                  void (async () => {
                    try {
                      const prepared = await normalizeImageFile(file);
                      clearFileUrl();
                      const url = URL.createObjectURL(file);
                      fileUrl.current = url;
                      setDraft({
                        kind: "file",
                        filename: prepared.filename,
                        bytes: prepared.bytes,
                        src: url,
                        x: 50,
                        y: 50,
                      });
                      onError(null);
                    } catch (err) {
                      onError(String(err));
                    } finally {
                      setBusyId(null);
                    }
                  })();
                }}
              />
            </form>

            {note && <p className="hint">{note}</p>}

            <div className="cover-picker-body">
            {(searching && hits.length === 0) || hits.length > 0 ? (
              <div className="cover-grid" ref={gridRef} aria-busy={searching && hits.length === 0}>
                {searching && hits.length === 0
                  ? Array.from({ length: 6 }, (_, index) => (
                      <div key={index} className="cover-pending" />
                    ))
                  : null}
                {hits.map((hit) => (
                  <button
                    key={hit.id}
                    type="button"
                    disabled={busyId != null}
                    title={hit.credit}
                    onClick={() =>
                      setDraft({
                        kind: "stock",
                        id: hit.id,
                        src: assetUrl(libraryRoot, hit.thumb_relpath),
                        credit: hit.credit,
                        x: 50,
                        y: 50,
                      })
                    }
                  >
                    <img src={assetUrl(libraryRoot, hit.thumb_relpath)} alt="" />
                    <span>{hit.credit}</span>
                  </button>
                ))}
                {page < pageCount && hits.length > 0 && (
                  <div ref={sentinelRef} className="cover-more" />
                )}
                {loadingMore && <p className="cover-more">Loading more</p>}
              </div>
            ) : null}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
