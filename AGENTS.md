# ePNote — Agent Guide

## Project

Tauri 2.x desktop app: Vue 3 + TypeScript (Vite 7) frontend, Rust backend with SQLite (rusqlite bundled). A wrong-question review system (错题本).

## Essential commands (run from repo root)

| Command | Purpose |
|---|---|
| `npm run dev` | Vite frontend-only dev (no Rust) |
| `npm run tauri dev` | Full Tauri dev (Vite + Rust, runs `beforeDevCommand: npm run dev`) |
| `npm run build` | `type-check` → `vite build` (via npm-run-all2) |
| `npm run type-check` | `vue-tsc --build` (also callable as `npm run typecheck`) |
| `npm run lint` | ESLint flat config, `--fix --cache` |
| `cargo test` | Rust unit tests (inline `#[cfg(test)] mod tests`) |
| `cargo check` | Rust compilation check |
| `npm run tauri build` | Release build (calls `beforeBuildCommand: npm run build`) |

Always run `cargo check` and `npm run type-check` before committing Rust changes.

## Architecture

- **Entry points**: `src/main.ts` (Vue), `src-tauri/src/main.rs` (Tauri builder + command registration), `src-tauri/src/lib.rs` (module tree)
- **Frontend routes** (`src/router/index.ts`): Protected by `meta: { requiresInit: true }` guard; `/init` is unprotected
- **Startup flow**: `main.ts` calls `tauri_check_init_default` → if initialized, loads settings and goes to `/review`; else `/init`
- **Mock data**: `src/mock/data.ts` used when Tauri runtime is unavailable (dynamic import fallback in `mockInit`)
- **Frontend-backend bridge**: `src/api/` wrappers around `@tauri-apps/api/core` `invoke()`; shim types in `src/shims-tauri.d.ts`
- **Rust layers** (bottom-up): `db/` (connection, migration) → `dao/` (data access) → `repo/` (typed queries) → `server/` (business logic) → `command/` (Tauri command handlers)
- **State machine**: `src-tauri/src/domain/state_machine.rs` — NEW→LEARNING→STABLE→DUE, SUSPENDED toggle; stable threshold = 3 consecutive correct
- **DB migrations** (`src-tauri/src/db/migrate.rs`): Versioned SQL migrations (v1-v9), auto-applied on `init_db()`
- **Dev views** at `/dev/center`, `/dev/recommendation-list`, `/dev/preview-recommendation` visible when `developerMode` is on (runtime-only, not persisted)

## Testing

- **Frontend**: No test framework configured (no vitest/jest).
- **Rust tests**: `cargo test`. Inline in source files under `#[cfg(test)]`. Test files exist in:
  - `src/domain/state_machine.rs` (state transition tests)
  - `src/util/time/tests.rs`
  - `src/db/tests.rs`
  - `src/repo/tests/` (4 files)
  - `src/asset/tests/` (4 files)
- Integration tests require a real Tauri environment.

## Configuration quirks

- `tsconfig.json` has project references: `tsconfig.app.json` (app code) + `tsconfig.node.json` (config files)
- ESLint config at `eslint.config.ts` (flat config), lints `**/*.{vue,ts,mts,tsx}`, ignores `dist/`, `dist-ssr/`, `coverage/`
- `.editorconfig`: 2-space indent, LF, UTF-8, max_line_length=100
- `vite.config.ts`: `@` alias → `./src`
- Tauri CSP: completely disabled (`csp: null`, `dangerousDisableAssetCspModification: true`)
- App config stored at `src-tauri/app_config.json` with single `root` path

## Notable

- No CI/GitHub Actions workflows present.
- Windows-only (current target). Requires Node >=20.19.0, Rust >=1.77.2.
- The `src/tauri-shims/` directory is empty; all Tauri shim declarations are in `src/shims-tauri.d.ts`.
- `src/mock/` data has a different QuestionState type (only NEW|LEARNING|STABLE, no DUE|SUSPENDED).
