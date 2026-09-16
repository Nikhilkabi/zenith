import { assetUrl } from "../lib/assets";
import type { Country } from "../types";

interface Props {
  country: Country;
  libraryRoot: string;
  onOpen: (countryId: string) => void;
}

export default function CountryCard({ country, libraryRoot, onOpen }: Props) {
  const initial = country.name.trim().charAt(0).toUpperCase() || "?";
  const count = country.place_count ?? 0;

  return (
    <article className="poster" onClick={() => onOpen(country.id)}>
      <div className={`poster-cover${country.cover_relpath ? "" : " typographic"}`}>
        {country.cover_relpath ? (
          <img
            src={assetUrl(libraryRoot, country.cover_relpath)}
            alt={country.name}
            loading="lazy"
          />
        ) : (
          <span>{initial}</span>
        )}
        <div className="poster-caption">
          <h3>{country.name}</h3>
          <p>
            {country.iso ? `${country.iso.toUpperCase()} · ` : ""}
            {count} {count === 1 ? "place" : "places"}
          </p>
        </div>
      </div>
    </article>
  );
}
