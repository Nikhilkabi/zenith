import { useCallback, useEffect, useState } from "react";
import { addLink, deleteLink, listLinks, looksLikeUrl, openUrl } from "../api";
import { assetUrl } from "../lib/assets";
import type { LinkRecord } from "../types";

interface Props {
  libraryRoot: string;
  placeId?: string;
  countryId?: string;
  onError: (message: string | null) => void;
  capturePaste?: boolean;
}

function domainOf(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return url;
  }
}

export default function Links({
  libraryRoot,
  placeId,
  countryId,
  onError,
  capturePaste = true,
}: Props) {
  const [links, setLinks] = useState<LinkRecord[]>([]);

  const refresh = useCallback(async () => {
    setLinks(await listLinks({ placeId, countryId }));
  }, [placeId, countryId]);

  useEffect(() => {
    refresh().catch((err) => onError(String(err)));
  }, [refresh, onError]);

  useEffect(() => {
    if (!capturePaste) return;

    function onPaste(e: ClipboardEvent) {
      const text = e.clipboardData?.getData("text")?.trim();
      if (!text || !looksLikeUrl(text)) return;
      const target = e.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.isContentEditable)
      ) {
        return;
      }
      e.preventDefault();
      const url = text.startsWith("http") ? text : `https://${text}`;
      addLink({ url, placeId, countryId })
        .then(() => refresh())
        .catch((err) => onError(String(err)));
    }

    window.addEventListener("paste", onPaste);
    return () => window.removeEventListener("paste", onPaste);
  }, [capturePaste, placeId, countryId, refresh, onError]);

  return (
    <section className="links">
      <h2 className="section-title">Links</h2>
      {links.length === 0 ? (
        <p className="links-empty">Paste a URL to add one.</p>
      ) : (
        <ul className="link-list">
          {links.map((link) => (
            <li key={link.id}>
              {link.thumb_relpath ? (
                <img src={assetUrl(libraryRoot, link.thumb_relpath)} alt="" />
              ) : (
                <div className="thumb blank" />
              )}
              <div className="meta">
                <a
                  href={link.url}
                  onClick={(e) => {
                    e.preventDefault();
                    openUrl(link.url).catch((err) => onError(String(err)));
                  }}
                >
                  {link.title || link.url}
                </a>
                <div className="domain">{domainOf(link.url)}</div>
              </div>
              <button
                type="button"
                className="remove text"
                aria-label="Remove link"
                onClick={() => {
                  deleteLink(link.id)
                    .then(refresh)
                    .catch((err) => onError(String(err)));
                }}
              >
                ×
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
