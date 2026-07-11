host_system := `rustc -vV | sed -n 's|host: ||p'`

on:
    glasgow run probe-rs -V 'A=3.3,B=5.0' --swclk 'A3' --swdio 'A2'

off:
    glasgow safe

reset:
    just off
    just on

flash-stock:
    probe-rs download --speed 1600 --chip AT32F415RCT7 --probe 20b7:9db1:C3-20251207T143255Z:1:0 --protocol swd --base-address "0x8000000" ~/Dropbox/egret-stuff/firmware/display_modified.bin --binary-format bin

build:
    cargo build --profile release-immediate-abort -Z build-std=core,alloc,panic_abort -Z build-std-features=compiler-builtins-mem,optimize_for_size

run:
    cargo run --profile release-immediate-abort -Z build-std=core,alloc,panic_abort -Z build-std-features=compiler-builtins-mem,optimize_for_size

docs PKG="scooter-display":
    RUSTDOCFLAGS="-Z unstable-options --sort-modules-by-appearance" cargo doc -Z build-std=core,alloc,panic_abort -Z build-std-features=compiler-builtins-mem,optimize_for_size --open --document-private-items -p {{PKG}}

test:
    cargo test --target {{host_system}} --no-default-features --features test

emulator:
    cargo run --release --bin scooter_emulator --no-default-features --features sim --target {{host_system}}

@binary:
    env DEFMT_LOG=off cargo objcopy --profile release-immediate-abort --no-default-features --features "prod-build" -Z build-std=core,alloc,panic_abort -Z build-std-features=compiler-builtins-mem,optimize_for_size -- -O binary firmware.bin

    printf "Firmware md5: "

    if command -v md5sum >/dev/null 2>&1; then \
        md5sum firmware.bin | awk '{print $1}'; \
    elif command -v md5 >/dev/null 2>&1; then \
        md5 -q firmware.bin; \
    else \
        echo "Error: Neither md5sum nor md5 command found." && exit 1; \
    fi

[positional-arguments]
@bloat *args="":
    cargo bloat --profile release-immediate-abort -Z build-std=core,alloc,panic_abort -Z build-std-features=compiler-builtins-mem,optimize_for_size "$@"

[positional-arguments]
@asm *args="":
    cargo asm --rust -Z build-std=core,alloc,panic_abort -Z build-std-features=compiler-builtins-mem,optimize_for_size --bin scooter_display "$@"

[positional-arguments]
@expand *args="":
    cargo expand -Z build-std=core,alloc,panic_abort -Z build-std-features=compiler-builtins-mem,optimize_for_size --lib "$@"
