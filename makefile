.PHONY: build clean default dist

SOURCE  := Cargo.toml $(wildcard src/*.rs)
TARGETS := \
	x86_64-unknown-linux-musl \
	x86_64-apple-darwin

default: build

build: target/debug/flightctl

target/debug/flightctl: $(SOURCE)
	cargo build

dist: $(foreach target,$(TARGETS),target/$(target)/release/flightctl)

target/%/release/flightctl: $(SOURCE)
	cargo build --release --target "$*"

clean:
	rm -rf target
