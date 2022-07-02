all: windows linux macos

windows:
	cargo build --target x86_64-pc-windows-gnu --features "rebuild-bindings"

linux:
	cargo build --target x86_64-unknown-linux-gnu --features "rebuild-bindings"

macos:
	cargo build --target x86_64-apple-darwin --features "rebuild-bindings"

local:
	cargo build --release --features "rebuild-bindings"