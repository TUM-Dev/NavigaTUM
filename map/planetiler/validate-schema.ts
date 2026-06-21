import Ajv2020 from "ajv/dist/2020.js";
import addFormats from "ajv-formats";
import {parse} from "yaml";
import {readFileSync} from "node:fs";

const config: string | undefined = process.argv[2];
if (!config) {
  console.error("Usage: node validate-schema.ts <config.yml>");
  process.exit(1);
}

// planetiler.schema.json $refs planetilerspec.schema.json for the `examples` key,
// so the spec schema must be registered before the config schema can compile.
const ajv = new Ajv2020({allErrors: true, strict: false});
addFormats.default(ajv);
ajv.addSchema(JSON.parse(readFileSync("planetilerspec.schema.json", "utf8")));
const schema: object = JSON.parse(readFileSync("planetiler.schema.json", "utf8"));

const doc: unknown = parse(readFileSync(config, "utf8"));
const validate = ajv.compile(schema);
if (!validate(doc)) {
  for (const error of validate.errors ?? []) {
    console.error(`${error.instancePath || "/"} ${error.message}`);
  }
  process.exit(1);
}
console.log(`${config} is valid against planetiler.schema.json`);
