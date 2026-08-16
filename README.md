# CS Interview Coach

A local-first desktop app for computer-science knowledge review: type a keyword, jump to the concept
card in **under a millisecond**, and drill it on an FSRS spaced-repetition schedule. Everything runs
offline — no server, no account, no telemetry.

Built with **Rust + Tauri 2** on the backend and **SvelteKit + TypeScript** on the front end.

```
┌──────────────┐   Tauri IPC    ┌─────────────────────────────────┐
│  SvelteKit   │ ─────────────► │  Rust core                      │
│  6 views     │ ◄───────────── │  ├─ fst      prefix autocomplete │
│  ⌘K palette  │                │  ├─ redb     embedded KV store   │
└──────────────┘                │  ├─ srs      FSRS scheduler      │
                                │  └─ import   bulk JSON ingest    │
                                └─────────────────────────────────┘
```

## Why it's built this way

**Sub-millisecond autocomplete.** Card titles and aliases are compiled into a
[finite-state transducer](https://blog.burntsushi.net/transducers/) (`fst` crate). The FST is a
minimal DAG over the sorted key set, so a prefix query walks one edge per character with no
allocation and no linear scan — lookup cost tracks query length, not corpus size.

**Embedded storage, not SQLite.** `redb` is a pure-Rust embedded KV store with MVCC and ACID
transactions. It avoids a C dependency in the Tauri bundle and keeps the write path a single
memory-mapped B-tree.

**FSRS over SM-2.** Review scheduling models memory as separate *difficulty* / *stability* /
*retrievability* terms rather than SM-2's single ease factor, so intervals adapt to per-card
history instead of a fixed multiplier.

**Structured cards, not free text.** Each concept is stored in 8 typed slots — definition,
mechanism, complexity, comparison, use case, exam points, pitfalls, code — so a card can be
rendered, diffed, and reviewed slot-by-slot instead of as an opaque blob.

## Features

| | |
|---|---|
| **6 navigation views** | Search · Map · Tree · Radial · Review · Jobs |
| **⌘K command palette** | Global instant recall from anywhere in the app |
| **FST autocomplete** | Sub-millisecond prefix completion over titles + aliases |
| **FSRS scheduling** | Difficulty/stability model with 4-grade review |
| **140+ knowledge cards** | DSA, ML/DL, LLM systems, RecSys, networking, distributed systems |
| **Bulk import** | JSON ingest pipeline with citation tracking per slot |
| **Job-filtered review** | Scope cards and reviews to a target role via `seed/jobs.yaml` |

## Getting started

```bash
npm install
npm run tauri dev      # launch the desktop app
npm run check          # typecheck
```

Bulk-load a card set:

```bash
./src-tauri/target/debug/cs-interview-coach import imports/03_github_new_concepts.json
```

Local data lives at `~/Library/Application Support/com.jyj.cs-interview-coach/coach.redb`.

## Layout

```
src-tauri/src/
  ├─ suggest.rs    FST index build + prefix query
  ├─ store.rs      redb persistence layer
  ├─ srs.rs        FSRS scheduler
  ├─ import.rs     JSON ingest + validation
  ├─ taxonomy.rs   topic tree loading
  ├─ commands.rs   Tauri IPC surface
  └─ models.rs     card / review / topic types
src/routes/        SvelteKit views (search, map, tree, radial, review, jobs)
seed/              topic taxonomy + job definitions
imports/           knowledge card sets (JSON)
docs/DESIGN.md     architecture and data-model design doc
```

## Roadmap

- Tantivy BM25 full-text search alongside FST prefix matching
- Local embeddings (BGE-M3) + HNSW index for semantic recall
- LLM follow-up dialogue grounded on the active card
- Automatic card synthesis from PDFs and notes, with conflict arbitration

## License

MIT
