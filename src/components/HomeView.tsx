import AddCountryForm from "./AddCountryForm";
import CountryCard from "./CountryCard";
import type { Country } from "../types";

interface Props {
  countries: Country[];
  libraryRoot: string;
  onCountryCreated: (country: Country) => void;
  onOpenCountry: (countryId: string) => void;
}

export default function HomeView({
  countries,
  libraryRoot,
  onCountryCreated,
  onOpenCountry,
}: Props) {
  return (
    <>
      <div className="masthead">
        <p className="eyebrow">Personal atlas</p>
        <h1 className="display">Countries</h1>
        <p className="lede">Posters for the places you keep saving and never file.</p>
      </div>

      {countries.length === 0 ? (
        <div className="empty-state">
          <h2>The wall is blank</h2>
          <p>Add a country, then dump Reels and stills into the inbox.</p>
          <AddCountryForm onCreated={onCountryCreated} />
        </div>
      ) : (
        <>
          <AddCountryForm onCreated={onCountryCreated} />
          <div className="poster-grid">
            {countries.map((country) => (
              <CountryCard
                key={country.id}
                country={country}
                libraryRoot={libraryRoot}
                onOpen={onOpenCountry}
              />
            ))}
          </div>
        </>
      )}
    </>
  );
}
