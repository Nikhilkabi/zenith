import { useState, type FormEvent } from "react";
import { createCountry } from "../api";
import type { Country } from "../types";

interface Props {
  onCreated: (country: Country) => void;
}

export default function AddCountryForm({ onCreated }: Props) {
  const [name, setName] = useState("");
  const [iso, setIso] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const country = await createCountry(name, iso.trim() || undefined);
      setName("");
      setIso("");
      onCreated(country);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <form onSubmit={handleSubmit}>
      {error && <div className="error-banner">{error}</div>}
      <div className="form-inline">
        <div className="form-row">
          <label htmlFor="country-name">Country name</label>
          <input
            id="country-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Japan"
            required
          />
        </div>
        <div className="form-row" style={{ maxWidth: 120 }}>
          <label htmlFor="country-iso">ISO</label>
          <input
            id="country-iso"
            value={iso}
            onChange={(e) => setIso(e.target.value)}
            placeholder="JP"
            maxLength={3}
          />
        </div>
        <button className="primary" type="submit" disabled={busy || !name.trim()}>
          {busy ? "Adding…" : "Add country"}
        </button>
      </div>
    </form>
  );
}
