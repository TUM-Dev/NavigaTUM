import "regenerator-runtime/runtime";

// For some reason this polyfill is not included automatically
if (typeof String.prototype.startsWith === 'undefined') {
  String.prototype.startsWith = function (needle) {
    return this.indexOf(needle) === 0;
  };
}

/*split*/

// polyfill webp support
const kTestImages = [
    "UklGRiIAAABXRUJQVlA4IBYAAAAwAQCdASoBAAEADsD+JaQAA3AAAAAA", // lossy
    "UklGRhoAAABXRUJQVlA4TA0AAAAvAAAAEAcQERGIiP4HAA==", // lossless
    "UklGRkoAAABXRUJQVlA4WAoAAAAQAAAAAAAAAAAAQUxQSAwAAAARBxAR/Q9ERP8DAABWUDggGAAAABQBAJ0BKgEAAQAAAP4AAA3AAP7mtQAAAA==", // alpha
    "UklGRlIAAABXRUJQVlA4WAoAAAASAAAAAAAAAAAAQU5JTQYAAAD/////AABBTk1GJgAAAAAAAAAAAAAAAAAAAGQAAABWUDhMDQAAAC8AAAAQBxAREYiI/gcA" // animation
];
let polyfill_webp_required=false;
for (const image of kTestImages){
    if (polyfill_webp_required)
        break;
    const img = new Image();
    img.onload = function () {
        if (img.width === 0 || img.height === 0) {
            polyfill_webp_required=true;
        }
    };
    img.onerror = function () {
        polyfill_webp_required=true;
    };
    img.src = "data:image/webp;base64," + image;
}
function polyfill_webp(){
    const head = document.getElementsByTagName("head")[0];
    const pl_js = document.createElement("script");
    pl_js.src = "/* @echo app_prefix */dist-cjs/polyfills.js";
    head.appendChild(pl_js);
    const webp_js = document.createElement("script");
    webp_js.src = "/* @echo app_prefix */dist-cjs/webp-hero.bundle.js";
    head.appendChild(webp_js);
}
if (polyfill_webp_required===true){
    console.warn("Your browser does not support webp images. We still support you, but we might drop this support in the future.")
    polyfill_webp();
}
else
    console.info("webp-polyfill mitigation not triggered")
