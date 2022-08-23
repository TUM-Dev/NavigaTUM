
export function setTitle(name: string) {
      document.title = `${name} – NavigaTUM`;
      document.querySelector('meta[property="og:title"]')?.setAttribute("content", name);
}
export function setDescription(description: string) {
  document.querySelector('meta[name="description"]')?.setAttribute("content", description);
  document.querySelector('meta[property="og:description"]')?.setAttribute("content", description);
}
export function setUrl() {
  document.querySelector('meta[property="og:url"]')?.setAttribute("content", window.location.href);
}
// Settings are also stored in localStorage to detect when setting
// a cookie did not work.
export function setLang (lang: string) {
  localStorage.setItem("lang", lang);
  document.cookie = `lang=${lang};Max-Age=31536000;SameSite=Lax;Path=/* @echo app_prefix */`;
  window.location.reload();
}
export function setTheme (theme: string) {
  localStorage.setItem("theme", theme);
  document.cookie = `theme=${theme};Max-Age=31536000;SameSite=Lax;Path=/* @echo app_prefix */`;
  window.location.reload();
}