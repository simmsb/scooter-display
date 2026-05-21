on:
    glasgow run probe-rs -V 'A=3.3,B=5.0' --swclk 'A3' --swdio 'A2'

off:
    glasgow safe

reset:
    just off
    just on

flash-stock:
    probe-rs download --speed 400 --chip AT32F415RCT7 --probe 20b7:9db1:C3-20251207T143255Z:1:0 --protocol swd --base-address "0x8000000" ~/Dropbox/egret-stuff/firmware/display_modified.bin --binary-format bin

build:
    cargo build --release -Z build-std=core,alloc,panic_abort -Z build-std-features=compiler-builtins-mem,optimize_for_size

run:
    cargo run --release -Z build-std=core,alloc,panic_abort -Z build-std-features=compiler-builtins-mem,optimize_for_size

docs PKG="scooter-display":
    RUSTDOCFLAGS="-Z unstable-options --sort-modules-by-appearance" cargo doc -Z build-std=core,alloc,panic_abort -Z build-std-features=compiler-builtins-mem,optimize_for_size --open --document-private-items -p {{PKG}}

test:
    cargo test --target aarch64-apple-darwin --no-default-features --features test

bloat:
    cargo bloat --release -Z build-std=core,alloc,panic_abort -Z build-std-features=compiler-builtins-mem,optimize_for_size

[positional-arguments]
@asm *args="":
    cargo asm --rust -Z build-std=core,alloc,panic_abort -Z build-std-features=compiler-builtins-mem,optimize_for_size --bin scooter_display "$@"

[positional-arguments]
@expand *args="":
    cargo expand -Z build-std=core,alloc,panic_abort -Z build-std-features=compiler-builtins-mem,optimize_for_size --lib "$@"
