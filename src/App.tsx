import { save } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  exportLibraryZip,
  getLibraryPath,
  inboxCount,
  listCountries,
  revealLibrary,
  search,
} from "./api";
import CountryView from "./components/CountryView";
import HomeView from "./components/HomeView";
import InboxView from "./components/InboxView";
import PlaceView from "./components/PlaceView";
import type { Country, SearchHit, View } from "./types";

export default function App() {
  const [countries, setCountries] = useState<Country[]>([]);
  const [libraryRoot, setLibraryRoot] = useState("");
  const [view, setView] = useState<View>({ kind: "home" });
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(0);
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHit[]>([]);

  const reload = useCallback(async () => {
    const [nextCountries, path, count] = await Promise.all([
      listCountries(),
      getLibraryPath(),
      inboxCount(),
    ]);
    setCountries(nextCountries);
    setLibraryRoot(path);
    setPending(count);
  }, []);

  useEffect(() => {
    reload().catch((err) => setError(String(err)));
  }, [reload]);

  useEffect(() => {
    if (!query.trim()) {
      setHits([]);
      return;
    }
    const handle = window.setTimeout(() => {
      search(query)
        .then(setHits)
        .catch((err) => setError(String(err)));
    }, 200);
    return () => window.clearTimeout(handle);
  }, [query]);

  const activeCountry = useMemo(
    () =>
      view.kind === "home" || view.kind === "inbox"
        ? null
        : countries.find((country) => country.id === view.countryId) ?? null,
    [countries, view],
  );

  async function handleExport() {
    const dest = await save({
      defaultPath: "Bucket-backup.zip",
      filters: [{ name: "Zip", extensions: ["zip"] }],
    });
    if (!dest || typeof dest !== "string") return;
    try {
      await exportLibraryZip(dest);
    } catch (err) {
      setError(String(err));
    }
  }

  function openHit(hit: SearchHit) {
    setQuery("");
    setHits([]);
    if (hit.kind === "country" && hit.country_id) {
      setView({ kind: "country", countryId: hit.country_id });
    } else if (hit.kind === "place" && hit.country_id) {
      setView({ kind: "place", placeId: hit.id, countryId: hit.country_id });
    } else {
      setView({ kind: "inbox" });
    }
  }

  return (
    <div className="app-shell">
      <aside className="spine" onClick={() => setView({ kind: "home" })}>
        <span>Bucket</span>
      </aside>
      <div className="page">
        <header className="topbar">
          <div className="search-wrap">
            <input
              className="search"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search countries, places, notes…"
            />
            {hits.length > 0 && (
              <ul className="search-results">
                {hits.map((hit) => (
                  <li key={`${hit.kind}-${hit.id}`}>
                    <button type="button" onClick={() => openHit(hit)}>
                      <span className="mono">{hit.kind}</span>
                      {hit.title}
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>
          <div className="topbar-actions">
            <button type="button" onClick={() => setView({ kind: "inbox" })}>
              Inbox{pending > 0 ? ` (${pending})` : ""}
            </button>
            <button type="button" onClick={() => void revealLibrary()}>
              Library folder
            </button>
            <button type="button" onClick={() => void handleExport()}>
              Export zip
            </button>
          </div>
        </header>

        <main className="content">
          {error && <div className="error-banner">{error}</div>}

          {view.kind === "home" && (
            <HomeView
              countries={countries}
              libraryRoot={libraryRoot}
              onCountryCreated={(country) =>
                setCountries((current) => [...current, country])
              }
              onOpenCountry={(countryId) => setView({ kind: "country", countryId })}
            />
          )}

          {view.kind === "inbox" && (
            <InboxView
              countries={countries}
              libraryRoot={libraryRoot}
              onFiled={() => void reload()}
              onBack={() => setView({ kind: "home" })}
            />
          )}

          {view.kind === "country" && activeCountry && (
            <CountryView
              country={activeCountry}
              libraryRoot={libraryRoot}
              onOpenPlace={(placeId) =>
                setView({
                  kind: "place",
                  placeId,
                  countryId: activeCountry.id,
                })
              }
              onBack={() => setView({ kind: "home" })}
              onCountryUpdated={(country) =>
                setCountries((current) =>
                  current.map((item) => (item.id === country.id ? country : item)),
                )
              }
              onCountryDeleted={() => {
                setCountries((current) =>
                  current.filter((item) => item.id !== activeCountry.id),
                );
                setView({ kind: "home" });
              }}
            />
          )}

          {view.kind === "country" && !activeCountry && <p>Country not found.</p>}

          {view.kind === "place" && (
            <PlaceView
              placeId={view.placeId}
              libraryRoot={libraryRoot}
              onBack={() => setView({ kind: "country", countryId: view.countryId })}
              onDeleted={() => {
                void reload();
                setView({ kind: "country", countryId: view.countryId });
              }}
            />
          )}
        </main>
      </div>
    </div>
  );
}
