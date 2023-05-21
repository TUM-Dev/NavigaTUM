export function saveCooke(name: string, value: string, reload = true) {
  localStorage.setItem(name, value);
  document.cookie = `${name}=${value};Max-Age=31536000;SameSite=Strict;Path=/`;
  alert(document.cookie);
  if (reload) window.location.reload();
}
