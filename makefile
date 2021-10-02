.PHONY: default build clean

default: build

build: target/debug/flightctl

target/debug/flightctl: src/*.rs
	cargo build

clean:
	rm -rf target
