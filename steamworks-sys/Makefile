all: windows linux macos

windows:
	cargo build --target x86_64-pc-windows-gnu --release --features "rebuild-bindings"

linux:
	cargo build --target x86_64-unknown-linux-gnu --release --features "rebuild-bindings"

macos:
	cargo build --target x86_64-apple-darwin --release --features "rebuild-bindings"

local:
	cargo build --release --features "rebuild-bindings"