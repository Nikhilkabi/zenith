import { useEffect, useMemo, useState } from "react";
import { getLibraryPath, listCountries, revealLibrary } from "./api";
import CountryView from "./components/CountryView";
import HomeView from "./components/HomeView";
import PlaceView from "./components/PlaceView";
import type { Country, View } from "./types";

export default function App() {
  const [countries, setCountries] = useState<Country[]>([]);
  const [libraryRoot, setLibraryRoot] = useState("");
  const [view, setView] = useState<View>({ kind: "home" });
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([listCountries(), getLibraryPath()])
      .then(([nextCountries, path]) => {
        setCountries(nextCountries);
        setLibraryRoot(path);
      })
      .catch((err) => setError(String(err)));
  }, []);

  const activeCountry = useMemo(
    () =>
      view.kind === "home"
        ? null
        : countries.find((country) => country.id === view.countryId) ?? null,
    [countries, view],
  );

  return (
    <div className="app-shell">
      <header className="topbar">
        <div className="brand" onClick={() => setView({ kind: "home" })}>
          Bucket
        </div>
        <div className="topbar-actions">
          {libraryRoot && (
            <button type="button" onClick={() => void revealLibrary()}>
              Open library folder
            </button>
          )}
        </div>
      </header>

      <main className="content">
        {error && <div className="error-banner">{error}</div>}
        {libraryRoot && (
          <p className="library-path" style={{ marginTop: 0 }}>
            Library: {libraryRoot}
          </p>
        )}

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
          />
        )}

        {view.kind === "country" && !activeCountry && <p>Country not found.</p>}

        {view.kind === "place" && (
          <PlaceView
            placeId={view.placeId}
            libraryRoot={libraryRoot}
            onBack={() => setView({ kind: "country", countryId: view.countryId })}
            onDeleted={() => setView({ kind: "country", countryId: view.countryId })}
          />
        )}
      </main>
    </div>
  );
}
