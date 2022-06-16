// polyfill webp support
// This file will be appended to core.js, but thus executed relatively late,
// because images are only loaded once vue finished initializing.

let webpUnpolyfilled = true;
function ensureWebpPolyfilled() {
  if (webpUnpolyfilled) {
    webpUnpolyfilled = false;
    console.warn(
      "Your browser does not support webp images. ",
      "We still support you, but we might drop this support in the future."
    );
    const head = document.getElementsByTagName("head")[0];
    const webpJS = document.createElement("script");
    webpJS.src = "/* @echo app_prefix */js/webp-hero.min.js";
    head.appendChild(webpJS);
  }
}
function testWebpFeature(image) {
  const img = new Image();
  img.onload = () => {
    if (img.width === 0 || img.height === 0) ensureWebpPolyfilled();
  };
  img.onerror = ensureWebpPolyfilled;
  img.src = `data:image/webp;base64,${image}`;
}

const webpTestImages = [
  "UklGRiIAAABXRUJQVlA4IBYAAAAwAQCdASoBAAEADsD+JaQAA3AAAAAA", // lossy
  "UklGRhoAAABXRUJQVlA4TA0AAAAvAAAAEAcQERGIiP4HAA==", // lossless
  "UklGRkoAAABXRUJQVlA4WAoAAAAQAAAAAAAAAAAAQUxQSAwAAAARBxAR/Q9ERP8DAABWUDggGAAAABQBAJ0BKgEAAQAAAP4AAA3AAP7mtQAAAA==", // alpha
];
webpTestImages.forEach((image) => {
  if (webpUnpolyfilled) testWebpFeature(image);
});
