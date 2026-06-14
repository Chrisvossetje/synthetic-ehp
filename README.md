# Synthetic EHP

An interactive viewer for the **synthetic EHP** spectral sequence. It also lets
you view the algebraic EHP (AEHP), the ordinary EHP, and their truncations, as
well as the stable AHSS.

The project has two parts:

- **`logic/`** — a Rust program that computes the spectral sequences from the
  Curtis tables and exports the result as TypeScript data files. See
  [`logic/README.md`](logic/README.md) for details.
- **`site/`** — a TypeScript + [d3](https://d3js.org/) web app that renders the
  exported data as an interactive chart. The rendering is based on the author's
  Ext resolver, which in turn draws on the [sseq](https://github.com/SpectralSequences)
  project / d3.

The Curtis table data originates from William Balderrama's
[website](https://williamb.info/lambda/classic-curtis-table.txt) (originally
computed by Tangora; algorithm due to Curtis) — see `logic/README.md` for the
full attribution.

## How it fits together

```
Curtis tables ──▶ logic/ (Rust) ──▶ site/src/data*.ts ──▶ site/ (TS + d3) ──▶ _site/
```

1. `logic/` parses the Curtis tables, solves the sequences, and writes
   `site/src/data.ts` (EHP) and `site/src/data_stable.ts` (AHSS).
2. esbuild bundles `site/src` into the static `_site/` folder.
3. `_site/` is served as a static site (locally, or via GitHub Pages on push to
   `main`).

The generated `data*.ts` files are committed, so you can build and run the site
without re-running the Rust step.

## Running the site

### Prerequisites
- Node.js + npm
- Python 3 (only to serve the built site locally)

### Build
```sh
npm install      # first time only
npm run build    # or: npm run minify   (minified)
```

To rebuild continuously while editing:
```sh
npm run watch
```
Note: `watch` only watches `site/src`, **not** `site/static`.

The site is built statically into the `_site/` folder.

### Serve
Because of CORS, serve the folder over HTTP rather than opening files directly:
```sh
cd ./_site && python3 -m http.server 8080
```
Then open <http://localhost:8080>.

### Type-check
```sh
npm run typecheck   # tsc -p ./site/src   (also `npm test`)
```

## What `npm run build` actually does
`npm run init` first removes the whole `_site/` folder and copies everything
from `site/static` into it. Then esbuild bundles and compiles the TypeScript
entry point (`site/src/main.ts`) into `_site/index.js` (code-split into
`_site/chunks/`, optionally minified).

## Regenerating the data
To recompute the spectral-sequence data, run the Rust engine — it writes the
`site/src/data*.ts` files in place:
```sh
cd logic && cargo run --release
```
See [`logic/README.md`](logic/README.md).

## Deployment
Pushing to `main` triggers `.github/workflows/deploy.yml`, which runs the
type-check, builds the minified site, and deploys `_site/` to GitHub Pages.
