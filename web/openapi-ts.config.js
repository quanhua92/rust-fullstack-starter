/** @type {import("openapi-typescript").ConfigFile} */
export default {
  input: "../docs/openapi.json",
  output: "src/types/api.ts",
  format: true,
  lint: false,
  transform: {
    // Ensure consistent property ordering for stable diffs
    sortKeys: true,
  },
  // Use consistent formatting options
  formatter: {
    // Configure to match Biome's preferences
    useTabs: true,
    tabWidth: 1,
    singleQuote: false,
    trailingComma: "es5",
  },
};