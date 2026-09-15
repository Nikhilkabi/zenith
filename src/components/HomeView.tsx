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
      <div className="section-header">
        <h1>Countries</h1>
      </div>

      {countries.length === 0 ? (
        <div className="empty-state">
          <h2>Start your travel scrapbook</h2>
          <p>Add your first country to begin collecting places and photos.</p>
          <AddCountryForm onCreated={onCountryCreated} />
        </div>
      ) : (
        <>
          <AddCountryForm onCreated={onCountryCreated} />
          <div className="grid" style={{ marginTop: "1.5rem" }}>
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
