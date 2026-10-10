import { useState } from "react";
import { createCountry, createGoal, createTrip, dropOnCountry, reorderCountries, reorderGoals, updateGoal, updateTrip } from "../api";
import { matchCountries } from "../data/countries";
import { normalizeImageFile } from "../lib/normalizeImage";
import { moveItem } from "../lib/reorder";
import type { Country, Goal, Trip } from "../types";
import FilterBar, { type StatusFilter } from "../ui/FilterBar";
import { AddTile, Tile, TileGrid } from "../ui/Tile";

interface Props {
  countries: Country[];
  goals: Goal[];
  trips: Trip[];
  libraryRoot: string;
  filter: StatusFilter;
  onFilter: (value: StatusFilter) => void;
  onCountriesChanged: (countries: Country[]) => void;
  onGoalsChanged: (goals: Goal[]) => void;
  onCountryCreated: (country: Country) => void;
  onGoalCreated: (goal: Goal) => void;
  onTripCreated: (trip: Trip) => void;
  onTripsChanged: (trips: Trip[]) => void;
  onOpenCountry: (countryId: string) => void;
  onOpenGoal: (goalId: string) => void;
  onOpenTrip: (tripId: string) => void;
  onError: (message: string | null) => void;
  onLibraryChanged: () => void;
}

export default function Home({
  countries,
  goals,
  trips,
  libraryRoot,
  filter,
  onFilter,
  onCountriesChanged,
  onGoalsChanged,
  onCountryCreated,
  onGoalCreated,
  onTripCreated,
  onTripsChanged,
  onOpenCountry,
  onOpenGoal,
  onOpenTrip,
  onError,
  onLibraryChanged,
}: Props) {
  const [ask, setAsk] = useState<Trip | null>(null);
  const year = new Date().getFullYear();
  const places = countries.reduce((n, c) => n + c.place_count, 0);
  const been = countries.reduce((n, c) => n + c.been_count, 0);
  const done = goals.filter((goal) => goal.status === "done").length;
  const visibleGoals = goals.filter((goal) => {
    if (filter === "dream") return goal.status === "dream";
    if (filter === "been") return goal.status === "done";
    return true;
  });
  const visible = countries.filter((country) => {
    if (filter === "dream") return country.been_count === 0;
    if (filter === "been") return country.been_count > 0;
    return true;
  });
  const tripGroups = [
    {
      label: "This year",
      trips: trips.filter((trip) => trip.status !== "done" && trip.year === year),
    },
    {
      label: "Someday",
      trips: trips.filter((trip) => trip.status !== "done" && trip.year !== year),
    },
    {
      label: "Been",
      trips: trips.filter((trip) => trip.status === "done"),
    },
  ].filter((group) => {
    if (group.trips.length === 0) return false;
    if (filter === "dream") return group.label !== "Been";
    if (filter === "been") return group.label === "Been";
    return true;
  });

  return (
    <>
      <div className="page-header">
        <p className="meta mast">
          {trips.length} trips · {goals.length} goals · {done} done · {countries.length} countries · {places} places · {been} been
        </p>
        <FilterBar value={filter} onChange={onFilter} doneLabel="Done" />
      </div>

      <div className={trips.length + goals.length + countries.length === 0 ? "home-empty" : undefined}>
      <section className="home-block">
        <h2 className="section-title">Trips</h2>
        {tripGroups.map((group) => (
          <div key={group.label} className="trip-group">
            <h3 className="section-title">{group.label}</h3>
            <TileGrid>
              {group.trips.map((trip) => (
                <Tile
                  key={trip.id}
                  id={trip.id}
                  name={trip.name}
                  coverRelpath={trip.cover_relpath}
                  coverX={trip.cover_x}
                  coverY={trip.cover_y}
                  libraryRoot={libraryRoot}
                  countLabel={trip.when_text ?? (trip.stop_count > 0 ? `${trip.stop_count} places` : undefined)}
                  been={trip.status === "done"}
                  markLabel="done"
                  onOpen={() => onOpenTrip(trip.id)}
                  onToggleBeen={() => {
                    if (trip.status === "done") {
                      updateTrip(trip.id, { status: "dream" })
                        .then((next) => onTripsChanged(trips.map((item) => (item.id === next.id ? next : item))))
                        .catch((err) => onError(String(err)));
                      return;
                    }
                    if (trip.stop_count === 0) {
                      updateTrip(trip.id, { status: "done", markPlaces: false })
                        .then((next) => onTripsChanged(trips.map((item) => (item.id === next.id ? next : item))))
                        .catch((err) => onError(String(err)));
                      return;
                    }
                    setAsk(trip);
                  }}
                />
              ))}
            </TileGrid>
          </div>
        ))}
        <TileGrid>
          <AddTile
            placeholder="Trip"
            onCreate={async (name) => {
              try {
                const trip = await createTrip(name);
                onTripCreated(trip);
                onOpenTrip(trip.id);
              } catch (err) {
                onError(String(err));
              }
            }}
          />
        </TileGrid>
      </section>

      <section className="home-block">
        <h2 className="section-title">Bucket list</h2>
        <TileGrid>
          {visibleGoals.map((goal) => (
            <Tile
              key={goal.id}
              id={goal.id}
              name={goal.name}
              coverRelpath={goal.cover_relpath}
              coverX={goal.cover_x}
              coverY={goal.cover_y}
              libraryRoot={libraryRoot}
              been={goal.status === "done"}
              markLabel="done"
              onOpen={() => onOpenGoal(goal.id)}
              onToggleBeen={() => {
                const status = goal.status === "done" ? "dream" : "done";
                updateGoal(goal.id, { status })
                  .then((next) => {
                    onGoalsChanged(goals.map((item) => (item.id === next.id ? next : item)));
                  })
                  .catch((err) => onError(String(err)));
              }}
              onReorder={(fromId, toId) => {
                const next = moveItem(goals, fromId, toId);
                onGoalsChanged(next);
                reorderGoals(next.map((goal) => goal.id)).catch((err) => onError(String(err)));
              }}
            />
          ))}
          <AddTile
            placeholder="Anything"
            onCreate={async (name) => {
              try {
                const goal = await createGoal(name);
                onGoalCreated(goal);
                onOpenGoal(goal.id);
              } catch (err) {
                onError(String(err));
              }
            }}
          />
        </TileGrid>
      </section>

      <section className="home-block">
        <h2 className="section-title">Destinations</h2>
        <TileGrid>
        {visible.map((country) => (
          <Tile
            key={country.id}
            id={country.id}
            name={country.name}
            coverRelpath={country.cover_relpath}
            coverX={country.cover_x}
            coverY={country.cover_y}
            libraryRoot={libraryRoot}
            countLabel={
              country.place_count > 0
                ? `${country.been_count} / ${country.place_count}`
                : undefined
            }
            onOpen={() => onOpenCountry(country.id)}
            onReorder={(fromId, toId) => {
              const next = moveItem(countries, fromId, toId);
              onCountriesChanged(next);
              reorderCountries(next.map((c) => c.id)).catch((err) =>
                onError(String(err)),
              );
            }}
            onDropFiles={(files) => {
              void (async () => {
                try {
                  for (const file of Array.from(files)) {
                    if (
                      !file.type.startsWith("image/") &&
                      !/\.(avif|jpe?g|png|webp)$/i.test(file.name)
                    ) {
                      continue;
                    }
                    const { filename, bytes } = await normalizeImageFile(file);
                    await dropOnCountry(country.id, filename, bytes);
                  }
                  onError(null);
                  onLibraryChanged();
                } catch (err) {
                  onError(String(err));
                }
              })();
            }}
          />
        ))}
        <AddTile
          placeholder="Country"
          suggestionsFor={(query) =>
            matchCountries(query).map((c) => ({ label: c.name, iso: c.iso }))
          }
          onCreate={async (name, iso) => {
            try {
              const country = await createCountry(name, iso);
              onCountryCreated(country);
              onOpenCountry(country.id);
            } catch (err) {
              onError(String(err));
            }
          }}
        />
        </TileGrid>
      </section>
      </div>

      {goals.length === 0 && countries.length === 0 && trips.length === 0 && (
        <p className="hint">
          Add a trip, a goal, or a country. Then give it a cover.
        </p>
      )}

      {ask && (
        <div className="sheet" role="dialog" aria-modal aria-label="Mark trip done" onClick={() => setAsk(null)}>
          <div className="sheet-card" onClick={(event) => event.stopPropagation()}>
            <h1 className="page-title">Mark {ask.name} done</h1>
            <p className="meta">Also mark the places on this trip as visited?</p>
            <div className="page-actions source-actions">
              <button
                type="button"
                className="primary"
                onClick={() => {
                  const trip = ask;
                  setAsk(null);
                  updateTrip(trip.id, { status: "done", markPlaces: true })
                    .then((next) => {
                      onTripsChanged(trips.map((item) => (item.id === next.id ? next : item)));
                      onLibraryChanged();
                    })
                    .catch((err) => onError(String(err)));
                }}
              >
                Yes, mark the places
              </button>
              <button
                type="button"
                className="text"
                onClick={() => {
                  const trip = ask;
                  setAsk(null);
                  updateTrip(trip.id, { status: "done", markPlaces: false })
                    .then((next) => onTripsChanged(trips.map((item) => (item.id === next.id ? next : item))))
                    .catch((err) => onError(String(err)));
                }}
              >
                No, only this trip
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
