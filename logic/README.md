# Synthetic EHP — computation engine

A Rust program (`logic`) that computes the **synthetic** spectral sequences
behind this project and exports the result for the website to render. It builds
both:

- the **AHSS** — the stable Adams–Novikov-style sequence (the "stable" model), and
- the **EHP** — the unstable EHP spectral sequence (the "main" model),

filling in every generator's state across pages together with the differentials
and τ-multiplications, and cross-checking the unstable EHP against the stable
AHSS in the metastable range.

The synthetic ("τ-linear") framing means each fact is tagged by its τ-power, so
the program tracks internal τ-multiplications (same bidegree) and external
τ-multiplications (same stem) alongside the differentials.

## Input: the Curtis tables

The starting point is a pair of (reduced) Curtis tables, which give full
information on all $H^*(\Lambda_n)$ and the algebraic differentials:

- `curtis_table.txt` — the unstable / EHP table
- `curtis_table_stable.txt` — the stable / AHSS table

These are copied from William Balderrama's
[website](https://williamb.info/lambda/classic-curtis-table.txt). The table was
originally computed by Martin C. Tangora in *"Computing the homology of the
lambda algebra"* (Memoirs of the AMS, Vol. 58, No. 337, 1985); the underlying
algorithm and name are due to Edward Curtis.

The algebraic EHP gives an upper bound for the generators (as
$\mathbb{Z}[\tau]$-modules) of the synthetic EHP: SEHP generators inject into
AEHP generators compatibly with the AEHP filtration and the Adams filtration.

The data is valid up to **stem 48** (`MAX_STEM` / `MAX_VERIFY_STEM` in
`src/main.rs`).

## How to run

Prerequisite: a recent Rust toolchain (the crate uses edition 2024).

```sh
cd logic
cargo run --release
```

`main` is a small scratch harness. As configured it replays the saved logs,
runs the automated EHP solver, verifies the result geometrically, and writes the
order table. Toggle the interactive vs. automated routines inside `main()` /
`routines.rs` during development.

### What it produces

Output paths are relative to the `logic/` working directory (i.e. written into
the repo root and into `site/src/`):

- `../site/src/data.ts` and `../site/src/data_stable.ts` — the TypeScript data
  files consumed by the website (EHP and AHSS respectively).
- `../log.{json,txt}`, `../log_stable.{json,txt}` (and `*_minimal` variants) —
  replayable **action logs**. A run reloads these first, so a session resumes
  exactly where it left off; reverting truncates the log.
- `../curtis_table*.txt` debug dumps and a LaTeX-style order table.

Discovered proofs save two views in the action log's existing string proof
field. The website's TypeScript export includes only the compact implication
chain, without the appended full tree. Existing proof strings without a full
tree section are exported unchanged.

- **Full proof tree:** every established choice in the refutation, its supporting
  subproofs, explored sibling results, and the bounded companion search. Shared
  subproofs use numbered references, retaining assumptions and step ordering.
- **Reduced proof tree:** `reduce_proof_tree(&full)` uses both branches to find
  common conclusions, then exports only the refutation root and its support.
  Shared conclusions and other differentials originating from the root
  assumption's source are removed, along with subproofs used only by those
  conclusions. Case splits whose every branch ends in the identical
  contradiction are replaced by that contradiction, including nested splits.
  Different errors and open or cancelled branches keep their case split.
  Conditional facts from nested guesses never become common facts merely because
  they were encountered. The JSON omitted-premise ledger links common facts to
  both original derivations in the unchanged full tree.

The reducer is a pure tree transformation; it does not rerun discovery. Export
the actual reduced graph as JSON, or render its remaining nodes as text:

```rust
let reduced = reduce_proof_tree(&full);
std::fs::write("reduced.txt", reduced.describe(&MODEL))?;
export_reduced_proof_tree(&full, std::fs::File::create("reduced.json")?)?;
```

These functions live in `solve::proof`. JSON choices contain generator IDs from
the same E1 model; the text renderer resolves their names. Reduced node IDs are
local to the exported graph, with `full_tree_id` linking back to the original.
The compact text starts directly with the root assumption, followed by `->`
lines for newly asserted facts and the resulting contradiction. It hides routine
exclusions and supporting refutations, which remain available in the graph and
full tree. Distinct terminal cases remain visible. The ledger and traversal
metadata are omitted from the text.

For each depth-zero binary discovery, the solver reruns the surviving world
from the same initial data, assuming that alternative without using the newly
proved verdict. The companion has fresh cancellation flags and is bounded by
both the rejected refutation's maximum successful depth and its furthest
traversal position (including supporting subproofs). Nested discoveries retain
full trees without starting further companions. If both alternatives contradict
current assumptions, that contradiction propagates upwards without choosing one.

Reduction changes only the displayed explanation. The full tree remains the
record of justification. Shared conclusions are not claimed to be a minimum
set of premises, and an undiscovered conclusion is not evidence of independence.
Existing saved proofs stay unchanged until their facts are rediscovered.

To check discovery performance, recompute through EHP stem 25 from the manual
facts (using the saved AHSS answer), without writing generated files:

```sh
cargo test --release benchmark_ehp_stem_25 -- --ignored --nocapture
```

Prefix the command with `EHP_SHOW_PROOF=1` to print both views of an example
stem 25 differential proof. The benchmark also checks all produced facts against
the saved verdicts.

## How it fits together (`src/`)

The crate is split into four areas, each with module-level docs:

- **`data/`** — static input and parsing. `curtis.rs` parses the Curtis tables
  into the algebraic E1 model and exposes the lazily-initialized
  `MODEL`/`DATA` statics (and their `STABLE_*` counterparts). `naming.rs` handles
  the `"tag[sphere]"` generator naming scheme; `static.rs` holds CSV comparison
  data and lookup tables.

- **`domain/`** — the core model of a synthetic spectral sequence.
  - `e1.rs` — the fixed E1 page: the list of generators and lookup indices.
  - `model.rs` — `SyntheticSS`, the asserted facts (differentials, internal and
    external τ-multiplications) layered on top of an E1 page.
  - `process.rs` — turns a `SyntheticSS` into computed pages by applying those
    facts page by page, reporting any `Issue`s.
  - `ss.rs` — `SSPages`, each generator's (AF, torsion) state across pages.

- **`solve/`** — the solving engine: verifying a partially-filled sequence
  against the known answers and searching for the facts that make it consistent.
  - `issues.rs` — the `Issue` enum and comparison routines.
  - `action.rs` — the `Action` log entries and how each is applied.
  - `ahss.rs` / `ehp.rs` — per-sequence issue finding.
  - `ehp_ahss.rs` — relating the unstable EHP sequence to the stable AHSS one.
  - `generate.rs` / `solve.rs` — proposing candidate facts and auto-deducing
    forced solutions.
  - `search.rs` — parallel speculative branch-and-bound primitives (rayon).
  - `proof.rs` — shared full proof trees and paired-search reduction.
  - `automated_ahss.rs` / `automated_ehp.rs` — the unattended solvers.

- **`io/`** — `cli.rs` (interactive terminal menu), `export.rs` (serialization
  to the website's `.ts` files, logs, and the order table), and `import.rs`
  (loading saved action logs back in).

`routines.rs` wires these together into the top-level entry points: the
`interactive_*` routines run the verify → resolve loop with a human at the
keyboard (auto-deducing what they can, prompting for the rest), while the
`automated_*` routines run the same loop fully unattended via the solvers. Both
replay a saved log first.
