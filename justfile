default:
    @just --list

preview version:
    git cliff --config cliff.toml --unreleased --tag v{{version}}

check:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --verbose
    cargo check --examples
    cargo doc --no-deps

prep version:
    @test "$(git rev-parse --abbrev-ref HEAD)" = "main" || (echo "❌ Release prep must run from main"; exit 1)
    @test -z "$(git status --short)" || (echo "❌ Working tree must be clean before release prep"; exit 1)
    @just check
    cargo set-version {{version}}
    git cliff --config cliff.toml --unreleased --tag v{{version}} --prepend CHANGELOG.md
    cargo publish --dry-run --locked --allow-dirty
    @echo ""
    @echo "✅ Done. Review CHANGELOG.md, then run: just commit {{version}}"

commit version:
    git add Cargo.toml Cargo.lock CHANGELOG.md
    git commit -m "chore(release): {{version}}"
    git tag -a v{{version}} -m "Release v{{version}}"
    @echo "✅ Tagged v{{version}}. Run: just push"

push:
    git push origin main --follow-tags
