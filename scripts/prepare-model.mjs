import { mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";

const contracts = readFileSync("specs/bench/data-model.dog", "utf8");
for (const [, file] of contracts.matchAll(/^entity: file:(.+)$/gm)) {
  if (!statSync(file).isFile()) {
    throw new Error(`Mapped source is not a regular file: ${file}`);
  }
}

mkdirSync(".doghouse/semantic", { recursive: true });
writeFileSync(".doghouse/semantic/bench.dog", contracts);
