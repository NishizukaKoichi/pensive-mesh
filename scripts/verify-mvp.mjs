import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";

const root = resolve(import.meta.dirname, "..");
const required = [
  "README.md",
  "SECURITY.md",
  "CONTRIBUTING.md",
  "docs/PRODUCT_SPEC.md",
  "docs/THREAT_MODEL.md",
  "docs/RECOVERY.md",
  "docs/VERIFICATION_REPORT.md",
  "schemas/context-pack-v1.json",
  "schemas/pensive-memory-event-v1.json",
  "schemas/spell-ticket-v1.json",
];

const missing = required.filter((path) => !existsSync(resolve(root, path)));
if (missing.length) {
  process.stderr.write(
    `Missing required v0.1 artifacts:\n${missing.join("\n")}\n`,
  );
  process.exit(1);
}

const cargo = readFileSync(resolve(root, "Cargo.toml"), "utf8");
for (const forbidden of ["reqwest", "lettre", "thirtyfour", "fantoccini"]) {
  if (cargo.includes(forbidden))
    throw new Error(`forbidden direct-action dependency: ${forbidden}`);
}

const test = spawnSync(
  "cargo",
  ["test", "-q", "-p", "pensive-core", "--test", "integration"],
  {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  },
);
if (test.status !== 0) {
  process.stderr.write(
    test.stderr || test.stdout || "integration acceptance failed\n",
  );
  process.exit(test.status ?? 1);
}

process.stdout.write(
  `${JSON.stringify({ version: "0.1.0", required_artifacts: required.length, integration_acceptance: "passed", external_models: "disabled", direct_actions: "absent" })}\n`,
);
