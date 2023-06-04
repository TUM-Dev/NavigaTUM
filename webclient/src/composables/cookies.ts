export function saveCooke(name: string, value: string, reload = true): void {
  localStorage.setItem(name, value);
  document.cookie = `${name}=${value};Max-Age=31536000;SameSite=Strict;Path=/`;
  if (reload) window.location.reload();
}
