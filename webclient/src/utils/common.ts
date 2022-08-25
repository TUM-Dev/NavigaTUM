export function setTitle(name: string) {
  document.title = `${name} – NavigaTUM`;
  document
    .querySelector('meta[property="og:title"]')
    ?.setAttribute("content", name);
}
export function setDescription(description: string) {
  document
    .querySelector('meta[name="description"]')
    ?.setAttribute("content", description);
  document
    .querySelector('meta[property="og:description"]')
    ?.setAttribute("content", description);
}
export function setUrl() {
  document
    .querySelector('meta[property="og:url"]')
    ?.setAttribute("content", window.location.href);
}
