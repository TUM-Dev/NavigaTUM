import type { LocationQueryValue } from "#vue-router";

export function setTitle(name: string): void {
  document.title = `${name} – NavigaTUM`;
  document.querySelector('meta[property="og:title"]')?.setAttribute("content", name);
}

export function setDescription(description: string): void {
  document.querySelector('meta[name="description"]')?.setAttribute("content", description);
  document.querySelector('meta[property="og:description"]')?.setAttribute("content", description);
}

export function firstOrDefault(
  value: LocationQueryValue | LocationQueryValue[] | undefined,
  defaultValue: string,
): string {
  if (Array.isArray(value)) return value[0] ?? defaultValue;
  return value ?? defaultValue;
}
