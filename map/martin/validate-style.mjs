import {validateStyleMin} from "@maplibre/maplibre-gl-style-spec";
import {readFileSync} from "fs";

const file = process.argv[2];
if (!file) {
  console.error("Usage: node validate-style.mjs <style.json>");
  process.exit(1);
}

const style = JSON.parse(readFileSync(file, "utf8"));
const errors = validateStyleMin(style);
if (errors.length) {
  for (const error of errors) {
    console.error(error.message);
  }
  process.exit(1);
}
