import {validateStyleMin, type ValidationError} from "@maplibre/maplibre-gl-style-spec";
import {readFileSync} from "fs";

const file: string | undefined = process.argv[2];
if (!file) {
  console.error("Usage: node validate-style.ts <style.json>");
  process.exit(1);
}

const style: unknown = JSON.parse(readFileSync(file, "utf8"));
const errors: ValidationError[] = validateStyleMin(style);
if (errors.length) {
  for (const error of errors) {
    console.error(error.message);
  }
  process.exit(1);
}
