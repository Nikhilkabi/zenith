import { useCallback, useEffect, useMemo, useRef, useState, type KeyboardEvent } from "react";
import {
  addTripStop,
  createCountry,
  createPlace,
  deleteTrip,
  getTrip,
  listAllPlaces,
  removeTripStop,
  reorderTripStops,
  setTripCost,
  updateTrip,
} from "../api";
import { matchCountries } from "../data/countries";
import { assetUrl } from "../lib/assets";
import type { Country, Place, TripDetail } from "../types";
import { TRIP_COSTS } from "../types";
import CoverImage from "../ui/CoverImage";
import InlineText from "../ui/InlineText";
import Mark from "../ui/Mark";
import Skeleton from "../ui/Skeleton";
import TwoClickDelete from "../ui/TwoClickDelete";
import CoverPicker, { CoverCredit } from "./CoverPicker";

interface Props {
  tripId: string;
  libraryRoot: string;
  countries: Country[];
  onOpenPlace: (placeId: string, countryId: string) => void;
  onDeleted: () => void;
  onChanged: () => void;
  onError: (message: string | null) => void;
}

function money(total: number, currency: string | null): string {
  const amount = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 }).format(total);
  return currency ? `${currency} ${amount}` : amount;
}

function digitsOf(raw: string): number | null {
  const cleaned = raw.replace(/[^\d]/g, "");
  if (!cleaned) return null;
  const value = Number(cleaned);
  if (!Number.isSafeInteger(value) || value > 1_000_000_000) return null;
  return value;
}

export default function TripView({
  tripId,
  libraryRoot,
  countries,
  onOpenPlace,
  onDeleted,
  onChanged,
  onError,
}: Props) {
  const [detail, setDetail] = useState<TripDetail | null>(null);
  const [whenText, setWhenText] = useState("");
  const [currency, setCurrency] = useState("");
  const [notes, setNotes] = useState("");
  const [amounts, setAmounts] = useState<Record<string, string>>({});
  const [picker, setPicker] = useState(false);
  const [adjusting, setAdjusting] = useState(false);
  const [ask, setAsk] = useState(false);
  const [adding, setAdding] = useState(false);
  const [catalog, setCatalog] = useState<Place[]>([]);
  const [placeQuery, setPlaceQuery] = useState("");
  const [newName, setNewName] = useState("");
  const [newCountry, setNewCountry] = useState("");
  const hydratedFor = useRef<string | null>(null);
  const whenTimer = useRef<number | null>(null);
  const currencyTimer = useRef<number | null>(null);
  const notesTimer = useRef<number | null>(null);
  const notesRef = useRef(notes);
  notesRef.current = notes;

  const refresh = useCallback(async () => {
    const next = await getTrip(tripId);
    setDetail(next);
    if (hydratedFor.current !== tripId) {
      setWhenText(next.trip.when_text ?? "");
      setCurrency(next.trip.currency ?? "");
      setNotes(next.trip.notes ?? "");
      const fields: Record<string, string> = {};
      for (const cost of next.costs) {
        fields[cost.category] = cost.amount == null ? "" : String(cost.amount);
      }
      setAmounts(fields);
      hydratedFor.current = tripId;
    }
    return next;
  }, [tripId]);

  useEffect(() => {
    refresh().catch((err) => onError(String(err)));
  }, [refresh, onError]);

  useEffect(() => {
    return () => {
      if (whenTimer.current != null) window.clearTimeout(whenTimer.current);
      if (currencyTimer.current != null) window.clearTimeout(currencyTimer.current);
      if (notesTimer.current != null) window.clearTimeout(notesTimer.current);
    };
  }, []);

  function applyDetail(next: TripDetail) {
    setDetail(next);
    const fields: Record<string, string> = {};
    for (const cost of next.costs) {
      fields[cost.category] = cost.amount == null ? "" : String(cost.amount);
    }
    setAmounts(fields);
  }

  async function saveWhen(value: string) {
    try {
      const trip = await updateTrip(tripId, { whenText: value });
      setDetail((current) => (current ? { ...current, trip } : current));
      onChanged();
    } catch (err) {
      onError(String(err));
    }
  }

  async function saveCurrency(value: string) {
    try {
      const trip = await updateTrip(tripId, { currency: value });
      setDetail((current) => (current ? { ...current, trip } : current));
      onChanged();
    } catch (err) {
      onError(String(err));
    }
  }

  async function saveNotes(value: string) {
    try {
      const trip = await updateTrip(tripId, { notes: value });
      setDetail((current) => (current ? { ...current, trip } : current));
    } catch (err) {
      onError(String(err));
    }
  }

  async function finish(markPlaces: boolean) {
    setAsk(false);
    try {
      const trip = await updateTrip(tripId, { status: "done", markPlaces });
      setDetail((current) => (current ? { ...current, trip } : current));
      onChanged();
    } catch (err) {
      onError(String(err));
    }
  }

  async function saveAmount(category: string, raw: string) {
    const amount = digitsOf(raw);
    const previous = detail?.costs.find((cost) => cost.category === category)?.amount ?? null;
    if (amount === previous) return;
    try {
      applyDetail(await setTripCost(tripId, category, amount));
      onChanged();
    } catch (err) {
      onError(String(err));
    }
  }

  function focusNextCost(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key !== "Enter") return;
    event.preventDefault();
    const inputs = Array.from(
      event.currentTarget.closest(".budget")?.querySelectorAll("input") ?? [],
    );
    const next = inputs[inputs.indexOf(event.currentTarget) + 1] as HTMLInputElement | undefined;
    if (next) next.focus();
    else event.currentTarget.blur();
  }

  async function createStop() {
    const placeName = newName.trim();
    const countryName = newCountry.trim();
    if (!placeName || !countryName) return;
    try {
      const existing = countries.find((item) => item.name.toLowerCase() === countryName.toLowerCase());
      const known = matchCountries(countryName).find(
        (item) => item.name.toLowerCase() === countryName.toLowerCase(),
      );
      const countryId =
        existing?.id ?? (await createCountry(known?.name ?? countryName, known?.iso)).id;
      const place = await createPlace(countryId, placeName);
      applyDetail(await addTripStop(tripId, place.id));
      setNewName("");
      setNewCountry("");
      setAdding(false);
      onChanged();
      onError(null);
    } catch (err) {
      onError(String(err));
    }
  }

  async function moveStop(index: number, direction: -1 | 1) {
    if (!detail) return;
    const next = detail.stops.slice();
    const target = index + direction;
    if (target < 0 || target >= next.length) return;
    const [item] = next.splice(index, 1);
    next.splice(target, 0, item);
    try {
      applyDetail(await reorderTripStops(tripId, next.map((stop) => stop.place_id)));
    } catch (err) {
      onError(String(err));
    }
  }

  const choices = useMemo(() => {
    const taken = new Set(detail?.stops.map((stop) => stop.place_id) ?? []);
    const needle = placeQuery.trim().toLowerCase();
    return catalog.filter((place) => {
      if (taken.has(place.id)) return false;
      if (!needle) return true;
      const country = countries.find((item) => item.id === place.country_id)?.name ?? "";
      return `${place.name} ${country}`.toLowerCase().includes(needle);
    });
  }, [catalog, countries, detail?.stops, placeQuery]);

  if (!detail) return <Skeleton count={4} />;

  const trip = detail.trip;
  const done = trip.status === "done";
  const labelFor = (category: string) =>
    TRIP_COSTS.find((item) => item.category === category)?.label ?? category;

  return (
    <div className="goal-page trip-page">
      <div className="goal-head">
        <div className="cover-band">
          {trip.cover_relpath ? (
            <CoverImage
              src={assetUrl(libraryRoot, trip.cover_relpath)}
              focusX={trip.cover_x}
              focusY={trip.cover_y}
            />
          ) : (
            <div className="cover-empty">
              <button type="button" className="text" onClick={() => setPicker(true)}>
                Choose a cover
              </button>
            </div>
          )}
        </div>
        <div className="goal-intro">
          <div className="title-row">
            <InlineText
              as="h1"
              className="page-title"
              value={trip.name}
              onSave={async (name) => {
                const next = await updateTrip(tripId, { name });
                setDetail((current) => (current ? { ...current, trip: next } : current));
                onChanged();
              }}
            />
            <Mark
              been={done}
              onLabel="done"
              onToggle={() => {
                if (done) {
                  updateTrip(tripId, { status: "dream" })
                    .then((next) => {
                      setDetail((current) => (current ? { ...current, trip: next } : current));
                      onChanged();
                    })
                    .catch((err) => onError(String(err)));
                  return;
                }
                if (detail.stops.length === 0) {
                  void finish(false);
                  return;
                }
                setAsk(true);
              }}
            />
          </div>
          <input
            className="when-line"
            value={whenText}
            placeholder="When — June 2027, or dry season"
            onChange={(event) => {
              const value = event.target.value;
              setWhenText(value);
              if (whenTimer.current != null) window.clearTimeout(whenTimer.current);
              whenTimer.current = window.setTimeout(() => void saveWhen(value), 500);
            }}
            onBlur={() => {
              if (whenTimer.current != null) window.clearTimeout(whenTimer.current);
              void saveWhen(whenText);
            }}
          />
          {trip.cover_credit && (
            <p className="cover-credit">
              <CoverCredit credit={trip.cover_credit} url={trip.cover_credit_url} />
            </p>
          )}
          <div className="page-actions">
            <button type="button" className="text" onClick={() => setPicker(true)}>
              Cover
            </button>
            {trip.cover_relpath && (
              <button
                type="button"
                className="text"
                onClick={() => {
                  setAdjusting(true);
                  setPicker(true);
                }}
              >
                Move
              </button>
            )}
            <TwoClickDelete
              label={trip.name}
              confirmLabel={`Delete ${trip.name}?`}
              onConfirm={async () => {
                await deleteTrip(tripId);
                onDeleted();
              }}
            />
          </div>
        </div>
      </div>

      {ask && (
        <div className="trip-ask">
          <p>Also mark the places on this trip as visited?</p>
          <div className="page-actions">
            <button type="button" className="primary" onClick={() => void finish(true)}>
              Yes, mark the places
            </button>
            <button type="button" className="text" onClick={() => void finish(false)}>
              No, only this trip
            </button>
          </div>
        </div>
      )}

      <div className="trip-spread">
      <section className="trip-block trip-places">
        <div className="trip-block-head">
          <h2 className="section-title">Places</h2>
          <button
            type="button"
            className="text"
            onClick={() => {
              setAdding(true);
              setPlaceQuery("");
              listAllPlaces().then(setCatalog).catch((err) => onError(String(err)));
            }}
          >
            Add a place
          </button>
        </div>
        {detail.stops.length === 0 ? (
          <p className="links-empty">Add a place you want to visit. It can be new, or one you already saved.</p>
        ) : (
          <ol className="stop-list">
            {detail.stops.map((stop, index) => (
              <li key={stop.place_id}>
                <button
                  type="button"
                  className="stop-name"
                  onClick={() => onOpenPlace(stop.place_id, stop.country_id)}
                >
                  <span>{stop.place_name}</span>
                  <span className="domain">{stop.country_name}</span>
                </button>
                <div className="page-actions">
                  <button
                    type="button"
                    className="text"
                    disabled={index === 0}
                    onClick={() => void moveStop(index, -1)}
                  >
                    Up
                  </button>
                  <button
                    type="button"
                    className="text"
                    disabled={index === detail.stops.length - 1}
                    onClick={() => void moveStop(index, 1)}
                  >
                    Down
                  </button>
                  <button
                    type="button"
                    className="text"
                    onClick={() => {
                      removeTripStop(tripId, stop.place_id)
                        .then((next) => {
                          applyDetail(next);
                          onChanged();
                        })
                        .catch((err) => onError(String(err)));
                    }}
                  >
                    Remove
                  </button>
                </div>
              </li>
            ))}
          </ol>
        )}
      </section>

      <section className="trip-block budget">
        <div className="budget-line">
          <h2 className="section-title">Rough cost</h2>
          <input
            className="currency-line"
            value={currency}
            placeholder="INR"
            aria-label="Currency"
            onChange={(event) => {
              const value = event.target.value;
              setCurrency(value);
              if (currencyTimer.current != null) window.clearTimeout(currencyTimer.current);
              currencyTimer.current = window.setTimeout(() => void saveCurrency(value), 500);
            }}
            onBlur={() => {
              if (currencyTimer.current != null) window.clearTimeout(currencyTimer.current);
              void saveCurrency(currency);
            }}
            onKeyDown={focusNextCost}
          />
        </div>
        {detail.costs.map((cost) => (
          <label key={cost.category} className="budget-line">
            <span>{labelFor(cost.category)}</span>
            <input
              inputMode="numeric"
              value={amounts[cost.category] ?? ""}
              placeholder="—"
              aria-label={labelFor(cost.category)}
              onChange={(event) => {
                const value = event.target.value.replace(/[^\d]/g, "");
                setAmounts((current) => ({ ...current, [cost.category]: value }));
              }}
              onBlur={() => void saveAmount(cost.category, amounts[cost.category] ?? "")}
              onKeyDown={focusNextCost}
            />
          </label>
        ))}
        <div className="budget-line budget-total">
          <span>Estimate</span>
          <span>{trip.cost_count > 0 ? money(trip.total, null) : "—"}</span>
        </div>
      </section>

      <textarea
        className="notes"
        value={notes}
        placeholder="Notes"
        rows={3}
        onChange={(event) => {
          const value = event.target.value;
          setNotes(value);
          if (notesTimer.current != null) window.clearTimeout(notesTimer.current);
          notesTimer.current = window.setTimeout(() => void saveNotes(value), 600);
        }}
        onBlur={() => {
          if (notesTimer.current != null) window.clearTimeout(notesTimer.current);
          void saveNotes(notesRef.current);
        }}
      />
      </div>

      {adding && (
        <div className="sheet" role="dialog" aria-modal aria-label="Add a place" onClick={() => setAdding(false)}>
          <div className="sheet-card wide" onClick={(event) => event.stopPropagation()}>
            <div className="page-header">
              <h1 className="page-title">Add a place</h1>
              <button type="button" className="text" onClick={() => setAdding(false)}>
                Close
              </button>
            </div>
            <form
              className="new-place"
              onSubmit={(event) => {
                event.preventDefault();
                void createStop();
              }}
            >
              <input
                className="when-line"
                value={newName}
                placeholder="Place, such as Hanoi"
                onChange={(event) => setNewName(event.target.value)}
              />
              <input
                className="when-line"
                value={newCountry}
                placeholder="Country, such as Vietnam"
                onChange={(event) => setNewCountry(event.target.value)}
              />
              <button type="submit" className="primary">
                Add to this trip
              </button>
            </form>
            {matchCountries(newCountry).slice(0, 4).length > 0 && newCountry.trim() && (
              <ul className="place-pick">
                {matchCountries(newCountry)
                  .slice(0, 4)
                  .map((item) => (
                    <li key={item.iso}>
                      <button type="button" onClick={() => setNewCountry(item.name)}>
                        <span>{item.name}</span>
                      </button>
                    </li>
                  ))}
              </ul>
            )}
            <input
              className="when-line"
              value={placeQuery}
              placeholder="Or search places you already saved"
              onChange={(event) => setPlaceQuery(event.target.value)}
            />
            {placeQuery.trim() && choices.length === 0 ? (
              <p className="links-empty">No saved places match that yet.</p>
            ) : choices.length > 0 ? (
              <ul className="place-pick">
                {choices.map((place) => {
                  const country = countries.find((item) => item.id === place.country_id)?.name;
                  return (
                    <li key={place.id}>
                      <button
                        type="button"
                        onClick={() => {
                          addTripStop(tripId, place.id)
                            .then((next) => {
                              applyDetail(next);
                              onChanged();
                              onError(null);
                            })
                            .catch((err) => onError(String(err)));
                        }}
                      >
                        <span>{place.name}</span>
                        {country && <span className="domain">{country}</span>}
                      </button>
                    </li>
                  );
                })}
              </ul>
            ) : null}
          </div>
        </div>
      )}

      {picker && (
        <CoverPicker
          libraryRoot={libraryRoot}
          initialQuery={trip.name}
          target="trip"
          ownerId={tripId}
          shape="wide"
          current={
            trip.cover_relpath
              ? { src: assetUrl(libraryRoot, trip.cover_relpath), x: trip.cover_x, y: trip.cover_y }
              : null
          }
          startOnCurrent={adjusting}
          onClose={() => {
            setPicker(false);
            setAdjusting(false);
          }}
          onApplied={async () => {
            await refresh();
            onChanged();
          }}
          onError={onError}
        />
      )}
    </div>
  );
}
