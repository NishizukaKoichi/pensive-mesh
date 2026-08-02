import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const result = spawnSync(
  "cargo",
  ["run", "-q", "-p", "pensive-core", "--example", "plaintext_probe"],
  {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  },
);

if (result.status !== 0) {
  process.stderr.write(
    result.stderr || result.stdout || "plaintext probe failed\n",
  );
  process.exit(result.status ?? 1);
}

const report = JSON.parse(result.stdout.trim());
if (
  report.plaintext_marker_found ||
  !report.sqlcipher_required ||
  report.files_scanned < 5
) {
  throw new Error(`invalid plaintext probe result: ${result.stdout}`);
}
process.stdout.write(`${JSON.stringify(report)}\n`);
