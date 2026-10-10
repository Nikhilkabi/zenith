import { useEffect, useState } from "react";

export const MONTHS = [
  "January",
  "February",
  "March",
  "April",
  "May",
  "June",
  "July",
  "August",
  "September",
  "October",
  "November",
  "December",
];

export function planLabel(month: number | null, year: number | null): string | null {
  if (month == null || year == null || month < 1 || month > 12) return null;
  return `${MONTHS[month - 1]} ${year}`;
}

interface Props {
  month: number | null;
  year: number | null;
  onSave: (month: number, year: number) => Promise<void>;
}

export default function PlanDate({ month, year, onSave }: Props) {
  const [monthText, setMonthText] = useState(month == null ? "" : String(month));
  const [yearText, setYearText] = useState(year == null ? "" : String(year));

  useEffect(() => {
    setMonthText(month == null ? "" : String(month));
    setYearText(year == null ? "" : String(year));
  }, [month, year]);

  function commit(nextMonth: string, nextYear: string) {
    const monthN = Number(nextMonth);
    const yearN = Number(nextYear);
    if (monthN < 1 || monthN > 12 || yearN < 1900 || yearN > 2199) return;
    if (monthN === month && yearN === year) return;
    void onSave(monthN, yearN);
  }

  return (
    <div className="plan-date">
      <select
        aria-label="Month"
        value={monthText}
        onChange={(event) => {
          setMonthText(event.target.value);
          commit(event.target.value, yearText);
        }}
      >
        <option value="">Month</option>
        {MONTHS.map((name, index) => (
          <option key={name} value={index + 1}>
            {name}
          </option>
        ))}
      </select>
      <input
        aria-label="Year"
        inputMode="numeric"
        placeholder="Year"
        value={yearText}
        onChange={(event) => setYearText(event.target.value.replace(/\D/g, "").slice(0, 4))}
        onBlur={() => commit(monthText, yearText)}
        onKeyDown={(event) => {
          if (event.key === "Enter") event.currentTarget.blur();
        }}
      />
    </div>
  );
}
