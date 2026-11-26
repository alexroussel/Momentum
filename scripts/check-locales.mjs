#!/usr/bin/env node
// Fails when a locale misses (or adds) keys compared to the reference locale `en` (ADR-012).
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../locales", import.meta.url));
const REFERENCE = "en";

function flatten(obj, prefix = "") {
  return Object.entries(obj).flatMap(([k, v]) =>
    v && typeof v === "object" ? flatten(v, `${prefix}${k}.`) : [`${prefix}${k}`],
  );
}

function keysOf(lang) {
  const dir = join(root, lang);
  return new Map(
    readdirSync(dir)
      .filter((f) => f.endsWith(".json"))
      .map((f) => [f, new Set(flatten(JSON.parse(readFileSync(join(dir, f), "utf8"))))]),
  );
}

const reference = keysOf(REFERENCE);
let problems = 0;
for (const lang of readdirSync(root).filter((l) => l !== REFERENCE)) {
  const other = keysOf(lang);
  for (const [file, refKeys] of reference) {
    const keys = other.get(file) ?? new Set();
    for (const k of refKeys)
      if (!keys.has(k)) (problems++, console.error(`${lang}/${file}: missing ${k}`));
    for (const k of keys)
      if (!refKeys.has(k)) (problems++, console.error(`${lang}/${file}: extra ${k}`));
  }
}
if (problems) process.exit(1);
console.log("locales: all keys match");
