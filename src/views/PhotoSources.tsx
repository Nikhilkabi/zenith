import { useEffect, useState } from "react";
import { getPhotoSources, openUrl, setPhotoSources } from "../api";

interface Props {
  onClose: () => void;
  onError: (message: string | null) => void;
}

export default function PhotoSources({ onClose, onError }: Props) {
  const [pexels, setPexels] = useState("");
  const [pixabay, setPixabay] = useState("");
  const [busy, setBusy] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    getPhotoSources()
      .then((sources) => {
        setPexels(sources.pexels);
        setPixabay(sources.pixabay);
      })
      .catch((err) => onError(String(err)));
  }, [onError]);

  async function save() {
    setBusy(true);
    setSaved(false);
    try {
      const next = await setPhotoSources(pexels, pixabay);
      setPexels(next.pexels);
      setPixabay(next.pixabay);
      setSaved(true);
      onError(null);
    } catch (err) {
      onError(String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="sheet" role="dialog" aria-modal aria-label="Photo sources" onClick={onClose}>
      <div className="sheet-card" onClick={(event) => event.stopPropagation()}>
        <div className="page-header">
          <h1 className="page-title">Photo sources</h1>
          <button type="button" className="text" onClick={onClose}>
            Close
          </button>
        </div>
        <p className="meta">
          Openverse works with no key. Pexels and Pixabay are free and need a key from their sites. The keys stay on this computer.
        </p>
        <div className="source-fields">
          <label>
            Pexels
            <input
              type="password"
              autoComplete="off"
              value={pexels}
              spellCheck={false}
              onChange={(event) => {
                setSaved(false);
                setPexels(event.target.value);
              }}
            />
            <button type="button" className="text" onClick={() => void openUrl("https://www.pexels.com/api/")}>
              Get a Pexels key
            </button>
          </label>
          <label>
            Pixabay
            <input
              type="password"
              autoComplete="off"
              value={pixabay}
              spellCheck={false}
              onChange={(event) => {
                setSaved(false);
                setPixabay(event.target.value);
              }}
            />
            <button type="button" className="text" onClick={() => void openUrl("https://pixabay.com/api/docs/")}>
              Get a Pixabay key
            </button>
          </label>
        </div>
        <div className="page-actions source-actions">
          <button type="button" className="primary" disabled={busy} onClick={() => void save()}>
            {busy ? "Saving" : "Save"}
          </button>
          {saved && <span className="meta">Saved</span>}
        </div>
      </div>
    </div>
  );
}
