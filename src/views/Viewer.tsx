import { useEffect, useState } from "react";
import { assetUrl } from "../lib/assets";
import type { ImageRecord } from "../types";

interface Props {
  images: ImageRecord[];
  index: number;
  libraryRoot: string;
  onClose: () => void;
  onIndex: (index: number) => void;
  onDelete: (imageId: string) => void | Promise<void>;
  onSetPlaceCover: (imageId: string) => void | Promise<void>;
  onSetCountryCover: (imageId: string) => void | Promise<void>;
  onCaption?: (imageId: string, caption: string) => void | Promise<void>;
}

export default function Viewer({
  images,
  index,
  libraryRoot,
  onClose,
  onIndex,
  onDelete,
  onSetPlaceCover,
  onSetCountryCover,
  onCaption,
}: Props) {
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [caption, setCaption] = useState("");
  const image = images[index];

  useEffect(() => {
    setConfirmDelete(false);
    setCaption(image?.caption ?? "");
  }, [index, image?.id, image?.caption]);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") {
        if (confirmDelete) setConfirmDelete(false);
        else onClose();
      } else if (e.key === "ArrowLeft") {
        onIndex((index - 1 + images.length) % images.length);
      } else if (e.key === "ArrowRight") {
        onIndex((index + 1) % images.length);
      } else if (e.key === "Delete") {
        if (confirmDelete) {
          void onDelete(image.id);
          setConfirmDelete(false);
        } else {
          setConfirmDelete(true);
        }
      } else if (e.key === "Enter" && confirmDelete) {
        void onDelete(image.id);
        setConfirmDelete(false);
      } else if (e.key === "c" || e.key === "C") {
        if (e.shiftKey) void onSetCountryCover(image.id);
        else void onSetPlaceCover(image.id);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [
    confirmDelete,
    image,
    images.length,
    index,
    onClose,
    onDelete,
    onIndex,
    onSetCountryCover,
    onSetPlaceCover,
  ]);

  if (!image) return null;

  return (
    <div className="viewer" role="dialog" aria-modal aria-label="Photo">
      <button type="button" className="viewer-close" onClick={onClose}>
        Close
      </button>
      <div className="viewer-stage">
        {images.length > 1 && (
          <button
            type="button"
            className="viewer-nav"
            aria-label="Previous photo"
            onClick={() => onIndex((index - 1 + images.length) % images.length)}
          >
            ‹
          </button>
        )}
        <img src={assetUrl(libraryRoot, image.display_relpath)} alt="" />
        {images.length > 1 && (
          <button
            type="button"
            className="viewer-nav"
            aria-label="Next photo"
            onClick={() => onIndex((index + 1) % images.length)}
          >
            ›
          </button>
        )}
      </div>
      <div className="viewer-dock">
        {onCaption && (
          <input
            className="viewer-caption"
            value={caption}
            placeholder="Caption"
            onChange={(e) => setCaption(e.target.value)}
            onBlur={() => {
              if ((image.caption ?? "") !== caption) {
                void onCaption(image.id, caption);
              }
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter") e.currentTarget.blur();
              e.stopPropagation();
            }}
          />
        )}
        {image.taken_at && <span className="viewer-date">{image.taken_at}</span>}
        <button type="button" onClick={() => void onSetPlaceCover(image.id)}>
          Set cover
        </button>
        <button type="button" onClick={() => void onSetCountryCover(image.id)}>
          Country cover
        </button>
        <button
          type="button"
          className="danger"
          onClick={() => {
            if (!confirmDelete) {
              setConfirmDelete(true);
              return;
            }
            void onDelete(image.id);
            setConfirmDelete(false);
          }}
        >
          {confirmDelete ? "Confirm delete" : "Delete"}
        </button>
      </div>
    </div>
  );
}
