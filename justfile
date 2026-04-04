# Run all checks (mirrors CI)
check:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cargo nextest run

# Format code
fmt:
    cargo fmt

# Run tests in watch mode
dev:
    bacon test
