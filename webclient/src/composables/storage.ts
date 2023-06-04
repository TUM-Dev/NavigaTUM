const MS_PER_HOUR = 3.6e6;
export function setLocalStorageWithExpiry<T>(key: string, value: T, ttl: number) {
  // ttl in hours
  const now = new Date();

  const item = {
    expiry: now.getTime() + ttl * MS_PER_HOUR,
    value,
  };
  localStorage.setItem(key, JSON.stringify(item));

  // "storage" usually fires only across tabs, this way we
  // force it to fire in this window as well
  const e = new Event("storage");
  window.dispatchEvent(e);
}

export function getLocalStorageWithExpiry<T>(key: string, defaultValue: T): T {
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

export function removeLocalStorage(key: string): void {
  localStorage.removeItem(key);
  const e = new Event("storage");
  window.dispatchEvent(e);
}
