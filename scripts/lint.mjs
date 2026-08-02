import { readdirSync, readFileSync } from "node:fs";
import { relative, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const scanRoots = ["apps", "crates", "packages", "schemas", "scripts"];
const excludedDirectories = new Set(["dist", "gen", "node_modules", "target"]);

function collectFiles(directory) {
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) {
      if (!excludedDirectories.has(entry.name))
        files.push(...collectFiles(path));
    } else if (entry.isFile()) {
      files.push(relative(root, path));
    }
  }
  return files;
}

const files = scanRoots.flatMap((directory) =>
  collectFiles(resolve(root, directory)),
);
files.sort();

const textFiles = files.filter((file) =>
  /\.(rs|ts|mjs|json|html|css)$/.test(file),
);
const violations = [];
const forbidden = [
  [/\bfetch\s*\(/, "browser/network fetch is forbidden in Pensive v0.1"],
  [/\bXMLHttpRequest\b/, "XMLHttpRequest is forbidden"],
  [/\bWebSocket\s*\(/, "WebSocket is forbidden"],
  [/\breqwest\s*::/, "direct HTTP client is forbidden"],
  [
    /std::process::Command|Command::new\s*\(/,
    "arbitrary shell execution is forbidden",
  ],
  [/\bTODO\b|\bFIXME\b|\bHACK\b/, "unfinished debt marker"],
  [/console\.log\s*\(/, "console logging can leak user context"],
];

for (const file of textFiles) {
  const content = readFileSync(resolve(root, file), "utf8");
  if (file.endsWith(".json")) {
    try {
      JSON.parse(content);
    } catch (error) {
      violations.push(`${file}: invalid JSON (${error.message})`);
    }
  }
  if (file === "scripts/lint.mjs") continue;
  for (const [pattern, reason] of forbidden) {
    if (pattern.test(content)) violations.push(`${file}: ${reason}`);
  }
}

const ui = readFileSync(resolve(root, "apps/desktop/src/main.ts"), "utf8");
if (!ui.includes("escapeHtml") || !ui.includes("escapeAttr")) {
  violations.push(
    "apps/desktop/src/main.ts: dynamic evidence must be escaped before rendering",
  );
}

if (violations.length) {
  process.stderr.write(`${violations.join("\n")}\n`);
  process.exit(1);
}

process.stdout.write(
  `Static safety lint passed (${textFiles.length} files under ${relative(root, root) || "."}).\n`,
);
