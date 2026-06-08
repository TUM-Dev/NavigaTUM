export function wallTimeToRfc3339(wall: string): string | null {
  const instant = new Date(wall);
  return Number.isNaN(instant.getTime()) ? null : instant.toISOString();
}
