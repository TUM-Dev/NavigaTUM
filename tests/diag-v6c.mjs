import { chromium } from "@playwright/test";
const CENTER = [11.670099, 48.266921];
const STYLE = {
  version: 8,
  sources: { "test-pois": { type: "geojson", data: { type: "FeatureCollection", features: [
    { type: "Feature", properties: { indoor: "toilet", is_male_toilet: true }, geometry: { type: "Point", coordinates: CENTER } },
  ] } } },
  layers: [{ id: "indoor-pois", type: "circle", source: "test-pois", paint: { "circle-radius": 24 } }],
};
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 720 }, locale: "de-DE" });
await page.route("https://nav.tum.de/martin/style/navigatum-basemap.json", (r) =>
  r.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(STYLE) })
);
await page.goto("http://localhost:3000/map", { waitUntil: "networkidle" });
await page.waitForTimeout(4000);
const info = await page.evaluate(() => {
  const el = document.elementFromPoint(640, 390);
  const spinner = document.querySelector("#map-browse")?.parentElement?.querySelector(".animate-spin, [class*=Spinner]");
  return {
    atPoint: el ? `${el.tagName}.${el.className?.toString?.().slice(0, 80)}` : null,
    canvasParentHTML: document.querySelector("#map-browse")?.parentElement?.firstElementChild?.outerHTML?.slice(0, 200),
  };
});
console.log(JSON.stringify(info, null, 1));
