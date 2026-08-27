# Reproduction Commands

Run from the three sibling repositories after replacing temporary local path
overrides with the recorded git revisions.

## Conservation

```powershell
Set-Location C:\Users\Q\code\conservation
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

## Institution

```powershell
Set-Location C:\Users\Q\code\institution
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

## Ecosim

```powershell
Set-Location C:\Users\Q\code\ecosim
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
uv run maturin develop
uv run pytest -q
uv run pyright
uv build
git diff --check
```

## Media-history gate

```powershell
git ls-files "*.pdf" "*.png"
git rev-list --objects --all | Select-String -Pattern '\.(pdf|png)$'
```

Both media commands must produce no paths in every affected repository. PDF
and PNG research artifacts remain ignored and outside package/repository
history.
