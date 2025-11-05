// Build bundel lokal CosmJS untuk halaman pro (tanpa CDN)
import { build } from 'esbuild';

await build({
  entryPoints: ['src/bundle-entry.js'],
  bundle: true,
  format: 'iife',
  globalName: 'Cosm', // diekspos sebagai window.Cosm
  sourcemap: false,
  minify: true,
  outfile: 'bundle.js',
});

console.log('✓ bundle.js dibuat (paxi-mint-pro/bundle.js). Buka bundle-loader.html.');

