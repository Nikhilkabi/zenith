import { useCallback, useEffect, useRef, useState } from "react";
import { importStockPhoto, searchCovers } from "../api";
import { assetUrl } from "../lib/assets";
import type { CoverHit } from "../types";

interface Props {
  placeId: string;
  placeName: string;
  libraryRoot: string;
  onClose: () => void;
  onImported: () => void | Promise<void>;
  onFiles: (files: FileList) => void;
  onError: (message: string | null) => void;
}

export default function PhotoAdd({
  placeId,
  placeName,
  libraryRoot,
  onClose,
  onImported,
  onFiles,
  onError,
}: Props) {
  const [query, setQuery] = useState(placeName);
  const [hits, setHits] = useState<CoverHit[]>([]);
  const [page, setPage] = useState(1);
  const [pageCount, setPageCount] = useState(1);
  const [searching, setSearching] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [added, setAdded] = useState<string[]>([]);
  const [note, setNote] = useState<string | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);
  const gridRef = useRef<HTMLDivElement>(null);
  const sentinelRef = useRef<HTMLDivElement>(null);
  const request = useRef(0);
  const loadingMoreRef = useRef(false);

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
    void runSearch(placeName);
  }, [placeName, runSearch]);

  useEffect(() => {
    const root = gridRef.current;
    const node = sentinelRef.current;
    if (!root || !node) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) void loadMore();
      },
      { root, rootMargin: "160px" },
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, [hits.length, loadMore]);

  async function addHit(hit: CoverHit) {
    if (busyId || added.includes(hit.id)) return;
    setBusyId(hit.id);
    try {
      await importStockPhoto(placeId, hit.id);
      setAdded((current) => [...current, hit.id]);
      await onImported();
      onError(null);
    } catch (err) {
      onError(String(err));
    } finally {
      setBusyId(null);
    }
  }

  return (
    <div className="sheet" role="dialog" aria-modal aria-label="Add photos" onClick={onClose}>
      <div
        className="sheet-card photo-add"
        onClick={(event) => event.stopPropagation()}
        onDragOver={(event) => event.preventDefault()}
        onDrop={(event) => {
          event.preventDefault();
          if (event.dataTransfer.files.length) onFiles(event.dataTransfer.files);
        }}
      >
        <div className="page-header">
          <div>
            <h1 className="page-title">Add photos</h1>
            <p className="meta">
              Openverse, Pexels, and Pixabay, or photos from your computer. Add the Pexels and Pixabay keys from the menu.
            </p>
          </div>
          <button type="button" className="text" onClick={onClose}>
            Close
          </button>
        </div>

        <form
          className="cover-search"
          onSubmit={(event) => {
            event.preventDefault();
            void runSearch(query);
          }}
        >
          <input
            value={query}
            placeholder="Search free photos"
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
            Your photos
          </button>
          <input
            ref={fileRef}
            type="file"
            accept="image/jpeg,image/png,image/webp,image/avif,.jpg,.jpeg,.png,.webp,.avif"
            multiple
            hidden
            onChange={(event) => {
              const files = event.target.files;
              event.target.value = "";
              if (files?.length) onFiles(files);
            }}
          />
        </form>

        <div className="photo-drop">Or drop photos here</div>
        {note && <p className="hint">{note}</p>}

        <div className="cover-picker-body">
          {(searching && hits.length === 0) || hits.length > 0 ? (
            <div className="cover-grid" ref={gridRef} aria-busy={searching && hits.length === 0}>
              {searching && hits.length === 0
                ? Array.from({ length: 6 }, (_, index) => <div key={index} className="cover-pending" />)
                : null}
              {hits.map((hit) => {
                const saved = added.includes(hit.id);
                return (
                  <button
                    key={hit.id}
                    type="button"
                    className={saved ? "added" : undefined}
                    disabled={busyId != null || saved}
                    title={hit.credit}
                    onClick={() => void addHit(hit)}
                  >
                    <img src={assetUrl(libraryRoot, hit.thumb_relpath)} alt="" />
                    <span>{saved ? "Added" : busyId === hit.id ? "Saving" : hit.credit}</span>
                  </button>
                );
              })}
              {page < pageCount && hits.length > 0 && <div ref={sentinelRef} className="cover-more" />}
              {loadingMore && <p className="cover-more">Loading more</p>}
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
}
