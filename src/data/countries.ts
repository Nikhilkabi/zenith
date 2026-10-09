export interface CountryOption {
  name: string;
  iso: string;
  aliases?: string[];
}

/** ISO 3166-1 countries and travel territories, English short names. */
const LINES = `
AF Afghanistan
AX Åland Islands
AL Albania
DZ Algeria
AS American Samoa
AD Andorra
AO Angola
AI Anguilla
AQ Antarctica
AG Antigua and Barbuda
AR Argentina
AM Armenia
AW Aruba
AU Australia
AT Austria
AZ Azerbaijan
BS Bahamas
BH Bahrain
BD Bangladesh
BB Barbados
BY Belarus
BE Belgium
BZ Belize
BJ Benin
BM Bermuda
BT Bhutan
BO Bolivia
BQ Caribbean Netherlands
BA Bosnia and Herzegovina
BW Botswana
BR Brazil
IO British Indian Ocean Territory
BN Brunei
BG Bulgaria
BF Burkina Faso
BI Burundi
CV Cabo Verde | cape verde
KH Cambodia
CM Cameroon
CA Canada
KY Cayman Islands
CF Central African Republic
TD Chad
CL Chile
CN China
CX Christmas Island
CC Cocos Islands | keeling
CO Colombia
KM Comoros
CG Congo
CD DR Congo | democratic republic of the congo | drc
CK Cook Islands
CR Costa Rica
CI Côte d'Ivoire | ivory coast | cote d'ivoire
HR Croatia
CU Cuba
CW Curaçao | curacao
CY Cyprus
CZ Czechia | czech republic | czech
DK Denmark
DJ Djibouti
DM Dominica
DO Dominican Republic
EC Ecuador
EG Egypt
SV El Salvador
GQ Equatorial Guinea
ER Eritrea
EE Estonia
SZ Eswatini | swaziland
ET Ethiopia
FK Falkland Islands
FO Faroe Islands
FJ Fiji
FI Finland
FR France
GF French Guiana
PF French Polynesia
TF French Southern Territories
GA Gabon
GM Gambia
GE Georgia
DE Germany
GH Ghana
GI Gibraltar
GR Greece
GL Greenland
GD Grenada
GP Guadeloupe
GU Guam
GT Guatemala
GG Guernsey
GN Guinea
GW Guinea-Bissau
GY Guyana
HT Haiti
VA Vatican City | holy see
HN Honduras
HK Hong Kong
HU Hungary
IS Iceland
IN India
ID Indonesia
IR Iran
IQ Iraq
IE Ireland
IM Isle of Man
IL Israel
IT Italy
JM Jamaica
JP Japan
JE Jersey
JO Jordan
KZ Kazakhstan
KE Kenya
KI Kiribati
KP North Korea
KR South Korea | korea
KW Kuwait
KG Kyrgyzstan
LA Laos
LV Latvia
LB Lebanon
LS Lesotho
LR Liberia
LY Libya
LI Liechtenstein
LT Lithuania
LU Luxembourg
MO Macao | macau
MG Madagascar
MW Malawi
MY Malaysia
MV Maldives
ML Mali
MT Malta
MH Marshall Islands
MQ Martinique
MR Mauritania
MU Mauritius
YT Mayotte
MX Mexico
FM Micronesia
MD Moldova
MC Monaco
MN Mongolia
ME Montenegro
MS Montserrat
MA Morocco
MZ Mozambique
MM Myanmar | burma
NA Namibia
NR Nauru
NP Nepal
NL Netherlands | holland
NC New Caledonia
NZ New Zealand
NI Nicaragua
NE Niger
NG Nigeria
NU Niue
NF Norfolk Island
MK North Macedonia | macedonia
MP Northern Mariana Islands
NO Norway
OM Oman
PK Pakistan
PW Palau
PS Palestine
PA Panama
PG Papua New Guinea
PY Paraguay
PE Peru
PH Philippines
PN Pitcairn Islands | pitcairn
PL Poland
PT Portugal
PR Puerto Rico
QA Qatar
RE Réunion | reunion
RO Romania
RU Russia
RW Rwanda
BL Saint Barthélemy | saint barthelemy | st barts
SH Saint Helena
KN Saint Kitts and Nevis
LC Saint Lucia
MF Saint Martin
PM Saint Pierre and Miquelon
VC Saint Vincent and the Grenadines
WS Samoa
SM San Marino
ST São Tomé and Príncipe | sao tome
SA Saudi Arabia
SN Senegal
RS Serbia
SC Seychelles
SL Sierra Leone
SG Singapore
SX Sint Maarten
SK Slovakia
SI Slovenia
SB Solomon Islands
SO Somalia
ZA South Africa
GS South Georgia
SS South Sudan
ES Spain
LK Sri Lanka
SD Sudan
SR Suriname
SJ Svalbard and Jan Mayen | svalbard
SE Sweden
CH Switzerland
SY Syria
TW Taiwan
TJ Tajikistan
TZ Tanzania
TH Thailand
TL Timor-Leste | east timor | timor
TG Togo
TK Tokelau
TO Tonga
TT Trinidad and Tobago
TN Tunisia
TR Türkiye | turkey
TM Turkmenistan
TC Turks and Caicos Islands
TV Tuvalu
UG Uganda
UA Ukraine
AE United Arab Emirates | uae
GB United Kingdom | uk | britain | great britain
US United States | usa | america
UY Uruguay
UZ Uzbekistan
VU Vanuatu
VE Venezuela
VN Vietnam | viet nam
VG British Virgin Islands
VI U.S. Virgin Islands | us virgin islands
WF Wallis and Futuna
EH Western Sahara
YE Yemen
ZM Zambia
ZW Zimbabwe
XK Kosovo
`.trim();

export const ISO_COUNTRIES: CountryOption[] = LINES.split("\n").map((line) => {
  const space = line.indexOf(" ");
  const iso = line.slice(0, space);
  const [name, ...aliases] = line.slice(space + 1).split(" | ");
  return {
    iso,
    name,
    aliases: aliases.length > 0 ? aliases : undefined,
  };
});

function haystack(country: CountryOption): string[] {
  return [country.name, country.iso, ...(country.aliases ?? [])].map((part) =>
    part.toLowerCase(),
  );
}

/** Every country when the query is empty; otherwise every match, best first. */
export function matchCountries(query: string): CountryOption[] {
  const q = query.trim().toLowerCase();
  if (!q) return ISO_COUNTRIES;

  const ranked = ISO_COUNTRIES.flatMap((country) => {
    const parts = haystack(country);
    let rank = 4;
    if (parts.some((part) => part === q)) rank = 0;
    else if (parts.some((part) => part.startsWith(q))) rank = 1;
    else if (parts.some((part) => part.includes(q))) rank = 2;
    else return [];
    return [{ country, rank }];
  });

  ranked.sort(
    (a, b) => a.rank - b.rank || a.country.name.localeCompare(b.country.name),
  );
  return ranked.map((item) => item.country);
}
