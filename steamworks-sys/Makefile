all: windows linux macos

clean:
	cargo clean

windows: clean
	cargo build --target x86_64-pc-windows-gnu --features "rebuild-bindings"

linux: clean
	cargo build --target x86_64-unknown-linux-gnu --features "rebuild-bindings"

macos: clean
	cargo build --target x86_64-apple-darwin --features "rebuild-bindings"

local: clean
	cargo build --features "rebuild-bindings"