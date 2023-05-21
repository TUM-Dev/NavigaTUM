export function saveCooke(name:string, value:string, reload = true) {
  localStorage.setItem(name, value);
  // in our staging environment, we want to set the cookie too
  // domain=.. is set on the main site
  // on staging Path=.. is set to allow for changing cookies
  let scoping = `Domain=${import.meta.env.VITE_APP_URL}`;
  if (window.location.host !== import.meta.env.VITE_APP_URL) {
    scoping = `Path=${window.location.origin}`;
  }
  document.cookie = `${name}=${value};Max-Age=31536000;SameSite=Lax;${scoping}`;
  if (reload) window.location.reload();
}
