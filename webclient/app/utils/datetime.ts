export function wallTimeToRfc3339(wall: string): string | null {
  const instant = new Date(wall);
  return Number.isNaN(instant.getTime()) ? null : instant.toISOString();
}

// Inverse of `wallTimeToRfc3339`: the instant as a browser-local `datetime-local` value.
export function rfc3339ToWallTime(iso: string): string | null {
  const instant = new Date(iso);
  if (Number.isNaN(instant.getTime())) return null;
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${instant.getFullYear()}-${pad(instant.getMonth() + 1)}-${pad(instant.getDate())}T${pad(instant.getHours())}:${pad(instant.getMinutes())}`;
}

// Campus events happen in Munich; their dates render in Europe/Berlin regardless of the visitor's zone.
const EVENT_TZ = "Europe/Berlin";

export function formatEventDateRange(startsAt: string, endsAt: string, locale: "de" | "en"): string {
  const start = new Date(startsAt);
  const end = new Date(endsAt);
  if (Number.isNaN(start.getTime()) || Number.isNaN(end.getTime())) return "";
  return new Intl.DateTimeFormat(locale === "de" ? "de-DE" : "en-GB", {
    timeZone: EVENT_TZ,
    day: "numeric",
    month: "short",
    year: "numeric",
  }).formatRange(start, end);
}
