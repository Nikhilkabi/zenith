import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  exportLibraryZip,
  getLibraryPath,
  getPlace,
  importLibraryZip,
  listCountries,
  listGoals,
  listTrips,
  revealLibrary,
  sweepOrphans,
} from "./api";
import type { Country, Goal, SearchHit, Trip, View } from "./types";
import type { StatusFilter } from "./ui/FilterBar";
import Menu from "./ui/Menu";
import CheatSheet from "./views/CheatSheet";
import PhotoSources from "./views/PhotoSources";
import CountryView from "./views/Country";
import GoalView from "./views/Goal";
import Home from "./views/Home";
import TripView from "./views/Trip";
import PlaceView from "./views/Place";
import Search from "./views/Search";
import Trash from "./views/Trash";

export default function App() {
  const [countries, setCountries] = useState<Country[]>([]);
  const [goals, setGoals] = useState<Goal[]>([]);
  const [trips, setTrips] = useState<Trip[]>([]);
  const [libraryRoot, setLibraryRoot] = useState("");
  const [view, setView] = useState<View>({ kind: "home" });
  const [error, setError] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [sheet, setSheet] = useState(false);
  const [sources, setSources] = useState(false);
  const [homeFilter, setHomeFilter] = useState<StatusFilter>("all");
  const searchRef = useRef<HTMLInputElement>(null);

  const onError = useCallback((message: string | null) => {
    setError(message);
  }, []);

  const reload = useCallback(async () => {
    const [nextCountries, nextGoals, nextTrips, path] = await Promise.all([
      listCountries(),
      listGoals(),
      listTrips(),
      getLibraryPath(),
    ]);
    setCountries(nextCountries);
    setGoals(nextGoals);
    setTrips(nextTrips);
    setLibraryRoot(path);
  }, []);

  useEffect(() => {
    reload().catch((err) => setError(String(err)));
  }, [reload]);

  useEffect(() => {
    setError(null);
  }, [view]);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      const target = e.target as HTMLElement | null;
      const typing =
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.isContentEditable);

      if ((e.key === "/" || (e.key === "k" && (e.ctrlKey || e.metaKey))) && !typing) {
        e.preventDefault();
        searchRef.current?.focus();
        searchRef.current?.select();
      } else if (e.key === "?" && !typing) {
        e.preventDefault();
        setSheet((open) => !open);
      } else if (e.key === "Escape" && sheet) {
        e.preventDefault();
        setSheet(false);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [sheet]);

  const activeCountry = useMemo(() => {
    if (view.kind !== "country" && view.kind !== "place") return null;
    return countries.find((c) => c.id === view.countryId) ?? null;
  }, [countries, view]);

  const activeGoal = useMemo(
    () => (view.kind === "goal" ? (goals.find((g) => g.id === view.goalId) ?? null) : null),
    [goals, view],
  );

  const activeTrip = useMemo(
    () => (view.kind === "trip" ? (trips.find((trip) => trip.id === view.tripId) ?? null) : null),
    [trips, view],
  );

  const [placeLabel, setPlaceLabel] = useState<string | null>(null);

  useEffect(() => {
    if (view.kind !== "place") {
      setPlaceLabel(null);
      return;
    }
    getPlace(view.placeId)
      .then((p) => setPlaceLabel(p.name))
      .catch(() => setPlaceLabel(null));
  }, [view]);

  async function handleExport() {
    try {
      await exportLibraryZip();
    } catch (err) {
      setError(String(err));
    }
  }

  function openHit(hit: SearchHit) {
    if (hit.kind === "country" && hit.country_id) {
      setView({ kind: "country", countryId: hit.country_id });
    } else if (hit.kind === "place" && hit.country_id) {
      setView({ kind: "place", placeId: hit.id, countryId: hit.country_id });
    } else if (hit.kind === "link" && hit.country_id) {
      setView({ kind: "country", countryId: hit.country_id });
    } else if (hit.kind === "goal") {
      setView({ kind: "goal", goalId: hit.id });
    } else if (hit.kind === "trip") {
      setView({ kind: "trip", tripId: hit.id });
    }
  }

  return (
    <div className="app">
      <header className="shell">
        <button
          type="button"
          className="wordmark"
          aria-label="Zenith"
          onClick={() => {
            void reload();
            setView({ kind: "home" });
          }}
        >
          Zenith
        </button>

        <nav className="crumb" aria-label="Breadcrumb">
          {view.kind !== "home" && view.kind !== "trash" && activeCountry && (
            <>
              <span className="sep">/</span>
              <button
                type="button"
                onClick={() =>
                  setView({ kind: "country", countryId: activeCountry.id })
                }
              >
                {activeCountry.name}
              </button>
            </>
          )}
          {view.kind === "goal" && (
            <>
              <span className="sep">/</span>
              <span>{activeGoal?.name ?? "…"}</span>
            </>
          )}
          {view.kind === "trip" && (
            <>
              <span className="sep">/</span>
              <span>{activeTrip?.name ?? "…"}</span>
            </>
          )}
          {view.kind === "trash" && (
            <>
              <span className="sep">/</span>
              <span>Trash</span>
            </>
          )}
          {view.kind === "place" && placeLabel && (
            <>
              <span className="sep">/</span>
              <span>{placeLabel}</span>
            </>
          )}
          {view.kind === "place" && !placeLabel && (
            <>
              <span className="sep">/</span>
              <span>…</span>
            </>
          )}
        </nav>

        <div className="shell-tools">
          <Search
            query={query}
            onQuery={setQuery}
            onOpen={openHit}
            onError={onError}
            inputRef={searchRef}
          />
          <Menu
            items={[
              {
                label: "Library folder",
                onClick: () => void revealLibrary().catch((e) => setError(String(e))),
              },
              {
                label: "Export zip",
                onClick: () => void handleExport(),
              },
              {
                label: "Import zip",
                onClick: () => {
                  importLibraryZip()
                    .then((n) => {
                      void reload();
                      if (n > 0) setError(null);
                    })
                    .catch((e) => setError(String(e)));
                },
              },
              {
                label: "Trash",
                onClick: () => setView({ kind: "trash" }),
              },
              {
                label: "Sweep unused files",
                onClick: () => {
                  sweepOrphans()
                    .then((n) => {
                      if (n > 0) setError(null);
                    })
                    .catch((e) => setError(String(e)));
                },
              },
              {
                label: "Photo sources",
                onClick: () => setSources(true),
              },
              {
                label: "Keys",
                onClick: () => setSheet(true),
              },
            ]}
          />
        </div>
      </header>

      {error && (
        <div className="error-line">
          <span>{error}</span>
          <button type="button" className="text" onClick={() => setError(null)}>
            ×
          </button>
        </div>
      )}

      <main className="content">
        <div className="view" key={JSON.stringify(view)}>
          {view.kind === "home" && (
            <Home
              countries={countries}
              goals={goals}
              trips={trips}
              libraryRoot={libraryRoot}
              filter={homeFilter}
              onFilter={setHomeFilter}
              onCountriesChanged={setCountries}
              onGoalsChanged={setGoals}
              onCountryCreated={(country) => {
                setError(null);
                setCountries((current) => [...current, country]);
              }}
              onGoalCreated={(goal) => {
                setError(null);
                setGoals((current) => [...current, goal]);
              }}
              onTripCreated={(trip) => {
                setError(null);
                setTrips((current) => [...current, trip]);
              }}
              onTripsChanged={setTrips}
              onOpenCountry={(countryId) =>
                setView({ kind: "country", countryId })
              }
              onOpenGoal={(goalId) => setView({ kind: "goal", goalId })}
              onOpenTrip={(tripId) => setView({ kind: "trip", tripId })}
              onError={onError}
              onLibraryChanged={() => void reload()}
            />
          )}

          {view.kind === "trash" && (
            <Trash onChanged={() => void reload()} onError={onError} />
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
              onCountryUpdated={(country) =>
                setCountries((current) =>
                  current.map((item) =>
                    item.id === country.id ? country : item,
                  ),
                )
              }
              onCountryDeleted={() => {
                setCountries((current) =>
                  current.filter((item) => item.id !== activeCountry.id),
                );
                setView({ kind: "home" });
              }}
              onCoverChanged={() => void reload()}
              onError={onError}
            />
          )}

          {view.kind === "country" && !activeCountry && (
            <p className="meta">Country not found.</p>
          )}

          {view.kind === "trip" && (
            <TripView
              tripId={view.tripId}
              libraryRoot={libraryRoot}
              countries={countries}
              onOpenPlace={(placeId, countryId) =>
                setView({ kind: "place", placeId, countryId })
              }
              onDeleted={() => {
                setTrips((current) => current.filter((item) => item.id !== view.tripId));
                setView({ kind: "home" });
              }}
              onChanged={() => void reload()}
              onError={onError}
            />
          )}

          {view.kind === "goal" && (
            <GoalView
              goalId={view.goalId}
              libraryRoot={libraryRoot}
              onDeleted={() => {
                setGoals((current) => current.filter((item) => item.id !== view.goalId));
                setView({ kind: "home" });
              }}
              onChanged={() => void reload()}
              onError={onError}
            />
          )}

          {view.kind === "place" && (
            <PlaceView
              placeId={view.placeId}
              countryId={view.countryId}
              libraryRoot={libraryRoot}
              onDeleted={() => {
                void reload();
                setView({ kind: "country", countryId: view.countryId });
              }}
              onLibraryChanged={() => void reload()}
              onError={onError}
            />
          )}
        </div>
      </main>

      {sheet && <CheatSheet onClose={() => setSheet(false)} />}
      {sources && (
        <PhotoSources
          onClose={() => setSources(false)}
          onError={onError}
        />
      )}
    </div>
  );
}
