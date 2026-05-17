#!/usr/bin/env node
import { execSync } from 'child_process';
import { existsSync, mkdirSync } from 'fs';
import { join } from 'path';

const root = process.cwd();
const outDir = join(root, 'src', 'bevy_wasm_pkg');

function run(cmd) {
  console.log('$', cmd);
  execSync(cmd, { stdio: 'inherit', cwd: './tracks-renderer' });
}

try {
  // ensure output dir exists
  if (!existsSync(outDir)) mkdirSync(outDir, { recursive: true });

  // add target
//   run('rustup target add wasm32-unknown-unknown');

  // build the renderer crate with wasm feature
  run('cargo build --target wasm32-unknown-unknown --release -p tracks-renderer --features wasm');

  // locate wasm
  const wasmPath = 'target/wasm32-unknown-unknown/release/tracks_renderer.wasm';
  // run wasm-bindgen
  run(`wasm-bindgen ${wasmPath} --out-dir ${outDir} --target web`);

  console.log('\nWASM build complete. Output in', outDir);
} catch (err) {
  console.error('Build failed:', err);
  process.exit(1);
}
