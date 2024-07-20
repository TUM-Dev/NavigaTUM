export function setTitle(name: string): void {
  document.title = `${name} – NavigaTUM`;
  document.querySelector('meta[property="og:title"]')?.setAttribute("content", name);
}
export function setDescription(description: string): void {
  document.querySelector('meta[name="description"]')?.setAttribute("content", description);
  document.querySelector('meta[property="og:description"]')?.setAttribute("content", description);
}
