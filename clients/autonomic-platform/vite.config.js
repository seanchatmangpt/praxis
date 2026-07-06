import { defineConfig, transformWithEsbuild } from 'vite';
import fs from 'node:fs';
import path from 'node:path';

// Repo root (this app lives at <repo>/clients/autonomic-platform).
const REPO = path.resolve(__dirname, '..', '..');

/**
 * Dev-server artifact bridge (the CLIENT_ADAPTER_CONTRACT read path).
 *
 * Mechanism: a configureServer middleware maps stable `/praxis-artifacts/*`
 * URLs onto real repo files. `server.fs.allow` is widened to the repo root so
 * the same files are also reachable via Vite's `/@fs/` scheme if needed.
 * Nothing is copied or synthesized: a missing source file returns 404 and the
 * adapter renders UNKNOWN. In a production deployment the same paths must be
 * served by whatever hosts the build output; `vite build` does not embed them.
 *
 *   /praxis-artifacts/receipt.json       -> .ggen-v2/receipt.json
 *   /praxis-artifacts/receipt-log.jsonl  -> .ggen-v2/receipt-log.jsonl
 *   /praxis-artifacts/registry.md        -> docs/releases/v26.7.6/BREED_ALGORITHM_REGISTRY.md
 *   /praxis-artifacts/plan.json          -> first target/plan_run/<run>/plan.json (sorted),
 *                                           wrapped as { ref, data } so the client
 *                                           knows which run it is looking at
 */
const STATIC_MAP = {
  '/praxis-artifacts/receipt.json': ['.ggen-v2/receipt.json', 'application/json'],
  '/praxis-artifacts/receipt-log.jsonl': ['.ggen-v2/receipt-log.jsonl', 'text/plain'],
  '/praxis-artifacts/registry.md': ['docs/releases/v26.7.6/BREED_ALGORITHM_REGISTRY.md', 'text/markdown'],
};

function praxisArtifacts() {
  return {
    name: 'praxis-artifacts',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const url = (req.url || '').split('?')[0];
        if (STATIC_MAP[url]) {
          const [rel, mime] = STATIC_MAP[url];
          const abs = path.join(REPO, rel);
          if (!fs.existsSync(abs)) { res.statusCode = 404; return res.end('absent'); }
          res.setHeader('Content-Type', mime);
          return res.end(fs.readFileSync(abs));
        }
        if (url === '/praxis-artifacts/plan.json') {
          const runDir = path.join(REPO, 'target', 'plan_run');
          const candidates = fs.existsSync(runDir)
            ? fs.readdirSync(runDir).sort()
                .map((d) => path.join(runDir, d, 'plan.json'))
                .filter((p) => fs.existsSync(p))
            : [];
          if (!candidates.length) { res.statusCode = 404; return res.end('absent'); }
          const ref = path.relative(REPO, candidates[0]);
          res.setHeader('Content-Type', 'application/json');
          return res.end(JSON.stringify({ ref, data: JSON.parse(fs.readFileSync(candidates[0], 'utf8')) }));
        }
        next();
      });
    },
  };
}


// Vite's built-in esbuild pass skips plain .js during `vite build`, so the
// JSX-in-.js convention needs an explicit pre-transform (standard Vite recipe).
function jsxInJs() {
  return {
    name: 'treat-js-as-jsx',
    enforce: 'pre',
    async transform(code, id) {
      if (!/\/src\/.*\.js$/.test(id)) return null;
      return transformWithEsbuild(code, id, { loader: 'jsx', jsx: 'automatic' });
    },
  };
}

export default defineConfig({
  plugins: [jsxInJs(), praxisArtifacts()],
  // Source files use JSX inside .js (the bundle's convention — see the header
  // of src/AutonomicPlatform.js). Tell esbuild to treat src .js as JSX.
  esbuild: { loader: 'jsx', include: /\.jsx?$/, jsx: 'automatic' },
  optimizeDeps: { esbuildOptions: { loader: { '.js': 'jsx' } } },
  server: { fs: { allow: [REPO] } },
});
