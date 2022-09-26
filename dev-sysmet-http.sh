#!/bin/bash
if ! command -v cargo-watch >/dev/null; then
	echo "cargo-watch not found, installing..."
	cargo install cargo-watch
fi

trap 'kill $CSS_WATCHER 2>/dev/null; kill $BROWSER_SYNC 2>/dev/null; exit' INT
root=bin/sysmet-http/

yarn run serve >/dev/null &
BROWSER_SYNC=$!

cd "$root/css" && yarn run watch >/dev/null &
CSS_WATCHER=$!

cargo watch \
	--watch "$root/src" \
	--watch "$root/Cargo.toml" \
	--watch "$root/css/exports" \
	-- cargo run -p sysmet-http -- --db test.db -v