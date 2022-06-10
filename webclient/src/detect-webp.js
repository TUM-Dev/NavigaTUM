// polyfill webp support
// This file will be appended to core.js, but thus executed relatively late,
// because images are only loaded once vue finished initializing.

const webpTestImages = [
  "UklGRiIAAABXRUJQVlA4IBYAAAAwAQCdASoBAAEADsD+JaQAA3AAAAAA", // lossy
  "UklGRhoAAABXRUJQVlA4TA0AAAAvAAAAEAcQERGIiP4HAA==", // lossless
  "UklGRkoAAABXRUJQVlA4WAoAAAAQAAAAAAAAAAAAQUxQSAwAAAARBxAR/Q9ERP8DAABWUDggGAAAABQBAJ0BKgEAAQAAAP4AAA3AAP7mtQAAAA==", // alpha
];
let webp_unpolyfilled = true;
for (const image of webpTestImages) {
  if (webp_unpolyfilled) test_webp_feature(image);
}

function test_webp_feature(image) {
  const img = new Image();
  img.onload = () => {
    if (img.width === 0 || img.height === 0) ensure_webp_polyfilled();
  };
  img.onerror = ensure_webp_polyfilled;
  img.src = `data:image/webp;base64,${image}`;
}

function ensure_webp_polyfilled() {
  if (webp_unpolyfilled) {
    webp_unpolyfilled = false;
    console.warn(
      "Your browser does not support webp images. ",
      "We still support you, but we might drop this support in the future."
    );
    const head = document.getElementsByTagName("head")[0];
    const webp_js = document.createElement("script");
    webp_js.src = "/* @echo app_prefix */js/webp-hero.min.js";
    head.appendChild(webp_js);
  }
}
