#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { spawnSync } from 'node:child_process';
import { parseArgs } from 'node:util';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';

const { values } = parseArgs({ options: {
  before: { type: 'string' },
  after: { type: 'string' },
  output: { type: 'string' },
  lazygit: { type: 'string' },
  runs: { type: 'string', default: '15' },
  gitconfig: { type: 'boolean', default: false },
  colored: { type: 'boolean', default: false },
  hyperlinks: { type: 'boolean', default: false },
  fixture: { type: 'string', multiple: true },
  mode: { type: 'string', multiple: true },
} });
if (!values.before || !values.after || !values.output) {
  throw new Error('Usage: node etc/performance/compare.mjs --before BIN --after BIN --output JSON [--lazygit REPO] [--gitconfig] [--runs 15]');
}
const runs = Number(values.runs);
if (!Number.isInteger(runs) || runs < 3) throw new Error('--runs must be an integer of at least 3');
const root = fileURLToPath(new URL('../..', import.meta.url));
const gitColor = values.colored ? '--color=always' : '--color=never';
const binaries = [values.before, values.after].map(p => path.resolve(p));
const env = { ...process.env, DELTA_FEATURES: '', DELTA_PAGER: 'cat' };
if (!values.gitconfig) {
  Object.assign(env, { GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: os.devNull, GIT_CONFIG_COUNT: '0' });
  delete env.BAT_THEME;
}
function git(repo, args) {
  const result = spawnSync('git', args, { cwd: repo, maxBuffer: 64 * 1024 * 1024 });
  if (result.status !== 0) throw new Error(String(result.stderr));
  return result.stdout;
}
function diff(filename, oldLines, newLines) {
  return Buffer.from(`diff --git a/${filename} b/${filename}\nindex 1111111..2222222 100644\n--- a/${filename}\n+++ b/${filename}\n@@ -1,${oldLines.length} +1,${newLines.length} @@\n${oldLines.map(s => '-' + s + '\n').join('')}${newLines.map(s => '+' + s + '\n').join('')}`);
}
const fixtures = [
  ['empty', Buffer.alloc(0)],
  ['tiny', diff('main.rs', ['fn main() { println!("hello"); }'], ['fn main() { println!("hello world"); }'])],
  ['unicode', diff('main.rs', ['let café = "你好 👩‍💻";\t// old'], ['let café = "こんにちは 👨‍💻";\t// new'])],
  ['long-lines', diff('data.txt', Array(8).fill('old_value, '.repeat(240)), Array(8).fill('new_value, '.repeat(240)))],
  ['unpaired', diff('data.txt', Array.from({ length: 25 }, (_, i) => `old_${i} `.repeat(20)), Array.from({ length: 25 }, (_, i) => `new_${i} `.repeat(20)))],
  ['rust-history', git(root, ['log', '-100', '-p', gitColor, 'HEAD'])],
];
const revisions = { delta: git(root, ['rev-parse', 'HEAD']).toString().trim() };
if (values.lazygit) {
  const repo = path.resolve(values.lazygit);
  revisions.lazygit = git(repo, ['rev-parse', 'HEAD']).toString().trim();
  fixtures.push(['go-small', git(repo, ['show', '--format=', gitColor, 'HEAD'])]);
  fixtures.push(['go-history', git(repo, ['log', '-50', '-p', gitColor, 'HEAD'])]);
}
const modes = [
  ['unified', []],
  ['side-by-side', ['--side-by-side']],
  ['no-syntax', ['--syntax-theme=none']],
];
for (const [selected, available] of [[values.fixture, fixtures], [values.mode, modes]]) {
  for (const name of selected || []) {
    if (!available.some(([item]) => item === name)) throw new Error(`Unknown selection: ${name}`);
  }
}
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
function render(binary, args, input, capture = false) {
  const result = spawnSync(binary, args, {
    cwd: root, env, input,
    stdio: ['pipe', capture ? 'pipe' : 'ignore', 'pipe'],
    maxBuffer: 64 * 1024 * 1024,
    timeout: 60_000,
  });
  if (result.error || result.status !== 0) throw new Error(`${binary}: ${result.error || result.stderr}`);
  return result.stdout;
}
function stats(samples) {
  const sorted = [...samples].sort((a, b) => a - b);
  const median = sorted[Math.floor(sorted.length / 2)];
  const deviations = sorted.map(x => Math.abs(x - median)).sort((a, b) => a - b);
  return { median_ms: median, mad_ms: deviations[Math.floor(deviations.length / 2)], samples_ms: samples };
}
const results = [];
for (const [fixture, input] of fixtures) {
  if (values.fixture && !values.fixture.includes(fixture)) continue;
  for (const [mode, extra] of modes) {
    if (values.mode && !values.mode.includes(mode)) continue;
    const args = ['--paging=never', '--width=100', '--true-color=always',
      ...(!values.gitconfig ? ['--no-gitconfig', '--dark'] : []),
      ...(values.hyperlinks ? ['--hyperlinks', '--hyperlinks-file-link-format=lazygit-edit://{path}:{line}'] : []), ...extra];
    const outputs = binaries.map(bin => render(bin, args, input, true));
    if (!outputs[0].equals(outputs[1])) throw new Error(`Output mismatch: ${fixture}, ${mode}`);
    for (let warmup = 0; warmup < 3; warmup++) {
      for (const bin of binaries) render(bin, args, input);
    }
    const times = [[], []];
    for (let run = 0; run < runs; run++) {
      // Alternate order so a warming CPU or changing clock does not favor one binary.
      for (const index of run % 2 ? [1, 0] : [0, 1]) {
        const start = performance.now();
        render(binaries[index], args, input);
        times[index].push(performance.now() - start);
      }
    }
    const [before, after] = times.map(stats);
    const change = 100 * (after.median_ms / before.median_ms - 1);
    results.push({ fixture, mode, input_bytes: input.length, input_sha256: sha256(input), output_bytes: outputs[0].length,
      args, output_identical: true, before, after, change_percent: change });
    process.stderr.write(`${fixture} ${mode}: ${before.median_ms.toFixed(2)} -> ${after.median_ms.toFixed(2)} ms (${change.toFixed(1)}%)\n`);
  }
}
fs.writeFileSync(values.output, JSON.stringify({
  created_at: new Date().toISOString(), platform: `${os.platform()} ${os.release()} ${os.arch()}`,
  cpu: os.cpus()[0]?.model, node: process.version, binaries, revisions,
  binary_sha256: binaries.map(bin => sha256(fs.readFileSync(bin))),
  gitconfig: values.gitconfig, colored: values.colored, hyperlinks: values.hyperlinks, runs, warmups: 3, results,
}, null, 2) + '\n');
console.log(path.resolve(values.output));
