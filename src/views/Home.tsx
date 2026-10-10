import { createCountry, createGoal, createTrip, dropOnCountry, reorderCountries, reorderGoals, updateGoal, updateTrip } from "../api";
import { matchCountries } from "../data/countries";
import { normalizeImageFile } from "../lib/normalizeImage";
import { moveItem } from "../lib/reorder";
import type { Country, Goal, Trip } from "../types";
import { planLabel } from "../ui/PlanDate";
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
  const year = new Date().getFullYear();
  const places = countries.reduce((n, c) => n + c.place_count, 0);
  const been = countries.reduce((n, c) => n + c.been_count, 0);
  const doneGoals = goals.filter((goal) => goal.status === "done");
  const openGoals = goals.filter((goal) => goal.status !== "done");
  const done = doneGoals.length;
  const visibleCountries = filter === "been" ? [] : countries;
  const openGroups = [
    {
      label: "This year",
      trips: trips.filter((trip) => trip.status !== "done" && trip.year === year),
    },
    {
      label: "Someday",
      trips: trips.filter((trip) => trip.status !== "done" && trip.year !== year),
    },
  ].filter((group) => group.trips.length > 0);
  const reel = new Map<number, { trips: Trip[]; goals: Goal[] }>();
  const yearBucket = (value: number) => {
    const found = reel.get(value);
    if (found) return found;
    const next = { trips: [] as Trip[], goals: [] as Goal[] };
    reel.set(value, next);
    return next;
  };
  for (const trip of trips) {
    if (trip.status !== "done") continue;
    yearBucket(trip.done_year ?? trip.year ?? year).trips.push(trip);
  }
  for (const goal of doneGoals) {
    yearBucket(goal.done_year ?? year).goals.push(goal);
  }
  const reelYears = [...reel.keys()].sort((a, b) => b - a);
  const showWork = filter !== "been";
  const showReel = filter !== "dream" && reelYears.length > 0;
  const emptyLibrary = trips.length + goals.length + countries.length === 0;

  const toggleTrip = (trip: Trip) => {
    if (trip.status === "done") {
      updateTrip(trip.id, { status: "dream" })
        .then((next) => {
          onTripsChanged(trips.map((item) => (item.id === next.id ? next : item)));
          onLibraryChanged();
        })
        .catch((err) => onError(String(err)));
      return;
    }
    updateTrip(trip.id, { status: "done", markPlaces: true })
      .then((next) => {
        onTripsChanged(trips.map((item) => (item.id === next.id ? next : item)));
        onLibraryChanged();
      })
      .catch((err) => onError(String(err)));
  };

  const toggleGoal = (goal: Goal) => {
    const status = goal.status === "done" ? "dream" : "done";
    updateGoal(goal.id, { status })
      .then((next) => onGoalsChanged(goals.map((item) => (item.id === next.id ? next : item))))
      .catch((err) => onError(String(err)));
  };

  const tripTile = (trip: Trip) => (
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
      onToggleBeen={() => toggleTrip(trip)}
    />
  );

  const goalTile = (goal: Goal, reorder: boolean) => (
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
      countLabel={planLabel(goal.month, goal.year) ?? undefined}
      onOpen={() => onOpenGoal(goal.id)}
      onToggleBeen={() => toggleGoal(goal)}
      onReorder={
        reorder
          ? (fromId, toId) => {
              const next = moveItem(goals, fromId, toId);
              onGoalsChanged(next);
              reorderGoals(next.map((item) => item.id)).catch((err) => onError(String(err)));
            }
          : undefined
      }
    />
  );

  return (
    <>
      <div className="page-header">
        <p className="meta mast">
          {trips.length} trips · {goals.length} goals · {done} done · {countries.length} countries · {places} places · {been} been
        </p>
        <FilterBar value={filter} onChange={onFilter} doneLabel="Done" />
      </div>

      {showReel && (
        <section className="home-block reel">
          {reelYears.map((reelYear) => {
            const group = reel.get(reelYear);
            if (!group) return null;
            return (
              <div key={reelYear} className="trip-group">
                <h2 className="section-title">{reelYear}</h2>
                <TileGrid>
                  {group.trips.map((trip) => tripTile(trip))}
                  {group.goals.map((goal) => goalTile(goal, false))}
                </TileGrid>
              </div>
            );
          })}
        </section>
      )}

      {filter === "been" && reelYears.length === 0 && (
        <p className="hint">Finished trips and goals gather here, one year at a time.</p>
      )}

      {showWork && (
        <div className={emptyLibrary ? "home-board" : undefined}>
          <section className="home-block">
            <h2 className="section-title">Trips</h2>
            {openGroups.map((group, index) => (
              <div key={group.label} className="trip-group">
                {openGroups.length > 1 && <h3 className="section-title">{group.label}</h3>}
                <TileGrid>
                  {group.trips.map((trip) => tripTile(trip))}
                  {index === openGroups.length - 1 && (
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
                  )}
                </TileGrid>
              </div>
            ))}
            {openGroups.length === 0 && (
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
            )}
          </section>

          <section className="home-block">
            <h2 className="section-title">Bucket list</h2>
            <TileGrid>
              {openGoals.map((goal) => goalTile(goal, true))}
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
              {visibleCountries.map((country) => (
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
                    reorderCountries(next.map((c) => c.id)).catch((err) => onError(String(err)));
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
      )}

      {showWork && goals.length === 0 && countries.length === 0 && trips.length === 0 && (
        <p className="hint">Add a trip, a goal, or a country. Then give it a cover.</p>
      )}
    </>
  );
}
