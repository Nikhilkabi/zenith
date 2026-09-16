import { useEffect, useState, type FormEvent } from "react";
import { addLink, deleteLink, listLinks } from "../api";
import { assetUrl } from "../lib/assets";
import type { LinkRecord } from "../types";

interface Props {
  libraryRoot: string;
  placeId?: string;
  countryId?: string;
}

export default function LinkPanel({ libraryRoot, placeId, countryId }: Props) {
  const [links, setLinks] = useState<LinkRecord[]>([]);
  const [url, setUrl] = useState("");
  const [title, setTitle] = useState("");
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    setLinks(await listLinks({ placeId, countryId }));
  }

  useEffect(() => {
    refresh().catch((err) => setError(String(err)));
  }, [placeId, countryId]);

  async function handleAdd(event: FormEvent) {
    event.preventDefault();
    setError(null);
    try {
      await addLink({ url, title: title || undefined, placeId, countryId });
      setUrl("");
      setTitle("");
      await refresh();
    } catch (err) {
      setError(String(err));
    }
  }

  return (
    <section className="link-panel">
      <h2>Saved links</h2>
      {error && <div className="error-banner">{error}</div>}
      <form onSubmit={handleAdd} className="form-inline">
        <div className="form-row">
          <label>URL</label>
          <input
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="https://"
            required
          />
        </div>
        <div className="form-row">
          <label>Title</label>
          <input
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Optional"
          />
        </div>
        <button className="primary" type="submit" disabled={!url.trim()}>
          Add link
        </button>
      </form>
      {links.length === 0 ? (
        <p className="muted">No links yet. File inbox items here, or paste one above.</p>
      ) : (
        <ul className="link-list">
          {links.map((link) => (
            <li key={link.id}>
              {link.thumb_relpath && (
                <img src={assetUrl(libraryRoot, link.thumb_relpath)} alt="" />
              )}
              <a href={link.url} target="_blank" rel="noreferrer">
                {link.title || link.url}
              </a>
              <button
                type="button"
                onClick={() => {
                  deleteLink(link.id)
                    .then(refresh)
                    .catch((err) => setError(String(err)));
                }}
              >
                Remove
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
