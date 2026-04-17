# Contributing

## Requirements

- Rust stable (≥ 1.80)
- [`just`](https://github.com/casey/just) — task runner
- [`cargo-nextest`](https://nexte.st) — test runner
- [`jj`](https://github.com/jj-vcs/jj) — required for integration tests
- [`bacon`](https://github.com/Canop/bacon) — optional, for watch mode (`just dev`)

## Setup

```sh
git clone https://github.com/omarkohl/jgl
cd jgl
cargo build
```

## Development workflow

```sh
just fmt        # format code
just check      # run all checks (mirrors CI): fmt, clippy, tests
just dev        # watch mode (requires bacon)
just integration  # run integration tests only
```

Or directly:

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo nextest run
```

## Tests

- **CLI tests** (`tests/cli.rs`) — black-box tests against the compiled binary
- **Integration tests** (`tests/integration.rs`) — spin up real jj repos via the test harness

## Linting conventions

Clippy is configured in `Cargo.toml` with `pedantic` and `nursery` groups enabled. The following are hard errors:

- `unwrap_used`, `expect_used`, `panic` — use `anyhow::Result` and `?` instead
- `unsafe_code`
- `dbg_macro`, `todo`, `unimplemented`

## Commit conventions

Use [Conventional Commits](https://www.conventionalcommits.org/) with these types:

| Type | When |
|------|------|
| `feat:` | new user-facing feature |
| `fix:` | bug fix |
| `refactor:` | internal restructuring, no behavior change |
| `docs:` | documentation only |
| `tests:` | test additions or changes |
| `chore:` | maintenance (deps, config, cleanup) |
| `ci:` | CI/CD changes |
| `dev:` | dev tooling or version bumps |

Keep messages short. Focus on *why*, not *what*.

## Before submitting

Run `just check` — it mirrors the CI pipeline exactly. PRs must pass CI.

## Releasing

1. Update `CHANGELOG.md` — add a `## [x.y.z] - YYYY-MM-DD` section with release notes.
2. Set the version in `Cargo.toml` (and run `cargo generate-lockfile` to update `Cargo.lock`).
3. Commit: `chore: release x.y.z`
4. Tag and push:
   ```sh
   git tag vx.y.z
   git push origin main vx.y.z
   ```
   Pushing the tag triggers the GitHub Actions release workflow, which builds binaries and creates a GitHub release.
5. Publish to crates.io:
   ```sh
   cargo publish
   ```
6. Set the next dev version in `Cargo.toml` (e.g. `x.y+1.0-dev`), update `Cargo.lock`, and commit: `dev: set version x.y+1.0-dev`.
