# AGENTS — Subly

Short entry for coding agents. Full graph: [docs/README.md](docs/README.md).

## Stack

Tauri v2 · Vue 3 · TypeScript · Rust · redb · Pinia · Tailwind v4

## Read first

| Doc | Use when |
|-----|----------|
| [docs/DOCS_PROFILE.md](docs/DOCS_PROFILE.md) | Paths, verification commands |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Layers, module registry |
| [docs/PROJECT_STRUCTURE.md](docs/PROJECT_STRUCTURE.md) | Find files (Vue + repo) |
| [docs/BACKEND_STRUCTURE.md](docs/BACKEND_STRUCTURE.md) | Rust / Tauri / commands / IPC |
| [docs/design/README.md](docs/design/README.md) | Subsystem behavior |

## Rules

- UI calls `src/services/*Client.ts` → Tauri `invoke`; business logic stays in `src-tauri/src/commands/`.
- Do not invent APIs — grep `#[tauri::command]` and client wrappers.
- Local-first: assume offline; sync is optional.
- Doc language: English (`doc_language` in profile).
- Version bumps: follow [.cursor/rules/app-release-versioning.mdc](.cursor/rules/app-release-versioning.mdc).

## Verify before PR

```bash
pnpm build
pnpm test:backend
pnpm test:restart-smoke
```

Optional widget JSON contract: `pnpm test:ios-widget`, `pnpm test:android-widget` (JDK 17 for Gradle).

Optional known persistence risks: `pnpm test:known-risks` — see [docs/restart-risk-matrix.md](docs/restart-risk-matrix.md).

## Doc agents

Provisioning: `doc-capture-existing-project` skill → `.cursor/agents/doc-graph-scan.md`, `doc-capture.md`.
