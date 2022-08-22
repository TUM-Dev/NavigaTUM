export function setLocalStorageWithExpiry(key: string, value: any, ttl: number) {
  // ttl in hours
  const now = new Date();

  const item = {
    value: value,
    expiry: now.getTime() + ttl * 3.6e6,
  };
  localStorage.setItem(key, JSON.stringify(item));

  // "storage" usually fires only across tabs, this way we
  // force it to fire in this window as well
  const e = new Event("storage");
  window.dispatchEvent(e);
}

export function getLocalStorageWithExpiry<T>(key: string, defaultValue: T | null = null): T | null {
  const itemStr = localStorage.getItem(key);
  if (!itemStr) return defaultValue;
  const item = JSON.parse(itemStr);
  const now = new Date();
  if (now.getTime() > item.expiry) {
    localStorage.removeItem(key);
    return defaultValue;
  }
  return item.value;
}

export function removeLocalStorage(key: string) {
  localStorage.removeItem(key);
  const e = new Event("storage");
  window.dispatchEvent(e);
}
