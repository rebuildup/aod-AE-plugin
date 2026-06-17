set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

build:
    cargo build -p aod-ae-utils

release:
    cargo build -p aod-ae-utils --release

check:
    cargo fmt --all -- --check
    cargo clippy --workspace
    cargo test

publish-dry:
    cargo publish --dry-run -p aod-ae-utils

publish:
    cargo publish -p aod-ae-utils
