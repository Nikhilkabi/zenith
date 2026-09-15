import { assetUrl } from "../lib/assets";
import type { Country } from "../types";

interface Props {
  country: Country;
  libraryRoot: string;
  onOpen: (countryId: string) => void;
}

export default function CountryCard({ country, libraryRoot, onOpen }: Props) {
  const initial = country.name.trim().charAt(0).toUpperCase() || "?";

  return (
    <article className="card" onClick={() => onOpen(country.id)}>
      <div className={`card-cover${country.cover_relpath ? "" : " typographic"}`}>
        {country.cover_relpath ? (
          <img
            src={assetUrl(libraryRoot, country.cover_relpath)}
            alt={country.name}
            loading="lazy"
          />
        ) : (
          <span>{initial}</span>
        )}
      </div>
      <div className="card-body">
        <h3>{country.name}</h3>
        {country.iso && <p>{country.iso.toUpperCase()}</p>}
      </div>
    </article>
  );
}
