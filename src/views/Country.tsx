import { useCallback, useEffect, useState } from "react";
import {
  coverCredit,
  createPlace,
  deleteCountry,
  importImageBytes,
  listCountryPhotos,
  listPlaces,
  reorderPlaces,
  updateCountry,
  updatePlace,
} from "../api";
import { assetUrl } from "../lib/assets";
import { normalizeImageFile } from "../lib/normalizeImage";
import { moveItem } from "../lib/reorder";
import type { Country, ImageRecord, Place } from "../types";
import FilterBar, { type StatusFilter } from "../ui/FilterBar";
import InlineText from "../ui/InlineText";
import Skeleton from "../ui/Skeleton";
import { AddTile, Tile, TileGrid } from "../ui/Tile";
import TwoClickDelete from "../ui/TwoClickDelete";
import CoverPicker, { CoverCredit } from "./CoverPicker";
import Links from "./Links";

interface Props {
  country: Country;
  libraryRoot: string;
  onOpenPlace: (placeId: string) => void;
  onCountryUpdated: (country: Country) => void;
  onCountryDeleted: () => void;
  onCoverChanged: () => void;
  onError: (message: string | null) => void;
}

export default function CountryView({
  country,
  libraryRoot,
  onOpenPlace,
  onCountryUpdated,
  onCountryDeleted,
  onCoverChanged,
  onError,
}: Props) {
  const [places, setPlaces] = useState<Place[] | null>(null);
  const [photos, setPhotos] = useState<ImageRecord[]>([]);
  const [filter, setFilter] = useState<StatusFilter>("all");
  const [picker, setPicker] = useState(false);
  const [adjusting, setAdjusting] = useState(false);
  const [credit, setCredit] = useState<string | null>(null);
  const [creditUrl, setCreditUrl] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const [nextPlaces, nextPhotos] = await Promise.all([
      listPlaces(country.id),
      listCountryPhotos(country.id),
    ]);
    setPlaces(nextPlaces);
    setPhotos(nextPhotos);
  }, [country.id]);

  useEffect(() => {
    refresh().catch((err) => onError(String(err)));
  }, [refresh, onError]);

  useEffect(() => {
    if (!country.cover_relpath) {
      setCredit(null);
      setCreditUrl(null);
      return;
    }
    coverCredit(country.cover_relpath)
      .then((next) => {
        setCredit(next?.credit ?? null);
        setCreditUrl(next?.credit_url ?? null);
      })
      .catch(() => {
        setCredit(null);
        setCreditUrl(null);
      });
  }, [country.cover_relpath]);

  if (!places) return <Skeleton count={4} />;

  const been = places.filter((p) => p.status === "been").length;
  const visible = places.filter((place) => {
    if (filter === "dream") return place.status === "dream";
    if (filter === "been") return place.status === "been";
    return true;
  });

  return (
    <div className="country-page">
      <div className="page-header">
        <div>
          <InlineText
            as="h1"
            className="page-title"
            value={country.name}
            onSave={async (name) => {
              const next = await updateCountry(country.id, { name });
              onCountryUpdated(next);
            }}
          />
          <p className="lede">Places live here. A trip is a plan that puts some of them in order.</p>
          <p className="meta">
            {places.length} places · {been} been
            {country.iso ? ` · ${country.iso}` : ""}
            {credit ? (
              <>
                {" · "}
                <CoverCredit credit={credit} url={creditUrl} />
              </>
            ) : null}
          </p>
        </div>
        <div className="page-actions">
          <button type="button" className="text" onClick={() => setPicker(true)}>
            Cover
          </button>
          {country.own_cover && (
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
          <FilterBar value={filter} onChange={setFilter} />
          <button type="button" className="text" onClick={() => window.print()}>
            Print
          </button>
          <TwoClickDelete
            label={country.name}
            confirmLabel={`Delete ${country.name}?`}
            onConfirm={async () => {
              await deleteCountry(country.id);
              onCountryDeleted();
            }}
          />
        </div>
      </div>

      <TileGrid>
        {visible.map((place) => (
          <Tile
            key={place.id}
            id={place.id}
            name={place.name}
            coverRelpath={place.cover_relpath}
            coverX={place.cover_x}
            coverY={place.cover_y}
            libraryRoot={libraryRoot}
            countLabel={
              place.when_text
                ? place.when_text
                : place.image_count > 0
                  ? String(place.image_count)
                  : undefined
            }
            been={place.status === "been"}
            onOpen={() => onOpenPlace(place.id)}
            onToggleBeen={() => {
              updatePlace(place.id, {
                status: place.status === "been" ? "dream" : "been",
              })
                .then((next) =>
                  setPlaces((current) =>
                    (current ?? []).map((p) => (p.id === next.id ? next : p)),
                  ),
                )
                .catch((err) => onError(String(err)));
            }}
            onReorder={(fromId, toId) => {
              const next = moveItem(places, fromId, toId);
              setPlaces(next);
              reorderPlaces(next.map((p) => p.id)).catch((err) =>
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
                    await importImageBytes(place.id, filename, bytes);
                  }
                  await refresh();
                  onError(null);
                } catch (err) {
                  onError(String(err));
                }
              })();
            }}
          />
        ))}
        <AddTile
          placeholder="Place"
          onCreate={async (name) => {
            const place = await createPlace(country.id, name);
            setPlaces((current) => [...(current ?? []), place]);
            onOpenPlace(place.id);
          }}
        />
      </TileGrid>

      {photos.length > 0 && (
        <div className="photo-strip" aria-label="Recent photos">
          {photos.map((image) => (
            <button
              key={image.id}
              type="button"
              onClick={() => onOpenPlace(image.place_id)}
            >
              <img
                src={assetUrl(libraryRoot, image.thumb_relpath)}
                alt=""
                loading="lazy"
              />
            </button>
          ))}
        </div>
      )}

      <Links libraryRoot={libraryRoot} countryId={country.id} onError={onError} />

      {picker && (
        <CoverPicker
          libraryRoot={libraryRoot}
          initialQuery={country.name}
          target="country"
          ownerId={country.id}
          shape="wide"
          current={
            country.cover_relpath
              ? {
                  src: assetUrl(libraryRoot, country.cover_relpath),
                  x: country.cover_x,
                  y: country.cover_y,
                }
              : null
          }
          startOnCurrent={adjusting}
          onClose={() => {
            setPicker(false);
            setAdjusting(false);
          }}
          onApplied={onCoverChanged}
          onError={onError}
        />
      )}
    </div>
  );
}
