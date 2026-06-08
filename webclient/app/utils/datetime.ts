// Turns a browser `datetime-local` value (a zoneless `YYYY-MM-DDTHH:MM` wall-clock string) into the
// RFC3339 contract the server's event validator parses
// (`server/src/routes/feedback/proposed_edits/addition/event.rs`). `datetime-local` represents the
// time the user picked in their own browser timezone, so `new Date` parses it in that zone and we
// hand the server the equivalent UTC instant (which it stores as UTC anyway). Returns `null` for
// malformed input so callers can surface a validation error.

export function wallTimeToRfc3339(wall: string): string | null {
  const instant = new Date(wall);
  return Number.isNaN(instant.getTime()) ? null : instant.toISOString();
}
