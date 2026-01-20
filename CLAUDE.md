# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a comodule resolver and visualizer that combines Rust computation with TypeScript/JavaScript frontend. The project uses WebAssembly to expose Rust algebraic computation functions to a web interface for resolving and visualizing comodules.

## Architecture

- **Rust Backend (`rust-logic/`)**: Contains the core comodule resolution logic compiled to WebAssembly
  - Uses the `comodules` crate for algebraic computations
  - Exposes functions like `resolve()` and `a0()` to JavaScript via wasm-bindgen
  - Supports finite fields Fp for primes 2, 3, 5, 7
- **TypeScript Frontend (`site/src/`)**: Web interface built with D3.js for visualization
  - `main.ts` handles UI interactions and calls Rust functions
  - `chart.ts` manages the D3.js visualization
  - `page.ts` handles JSON parsing of resolution results
  - `examples.ts` provides preset examples

## Development Commands

**Build the site:**
```bash
npm run build
```

**Continuous development with file watching:**
```bash
npm run watch
```
Note: Only watches `site/src`, NOT `site/static`

**Type checking:**
```bash
npm run typecheck
```

**Run tests:**
```bash
npm run test
```

**Serve the built site locally:**
```bash
cd ./_site && python3 -m http.server 8080
```
Then visit `localhost:8080`

## Build Process

The build process:
1. Removes `_site/` and `site/pkg/` directories
2. Copies `site/static/` to `_site/`
3. Builds Rust to WebAssembly with `wasm-pack build --target web`
4. Copies WebAssembly files to both `_site/pkg/` and `site/pkg/`
5. Bundles TypeScript with esbuild into `_site/index.js`

## Key Dependencies

- **Rust**: `comodules` crate, `wasm-bindgen`, `serde`
- **Frontend**: D3.js for visualization, KaTeX for math rendering, esbuild for bundling

## Working with the Code

- The main entry point is `site/src/main.ts`
- Rust functions are imported from `../pkg/rust_logic.js` after WebAssembly compilation
- The UI toggles between resolver input and visualization views
- Chart visualization is handled by the `Chart` class in `chart.ts`