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

export function copyCurrentLink() {
  // c.f. https://stackoverflow.com/a/30810322
  const textArea = document.createElement("textarea");
  textArea.value = window.location.href;

  // Avoid scrolling to bottom
  textArea.style.top = "0";
  textArea.style.left = "0";
  textArea.style.position = "fixed";

  document.body.appendChild(textArea);
  textArea.focus();
  textArea.select();

  let success = false;
  try {
    success = document.execCommand("copy");
  } catch (err) {
    console.error("Failed to copy to clipboard", err);
  }

  document.body.removeChild(textArea);
  return success;
}
