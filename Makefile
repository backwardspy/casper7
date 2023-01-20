.PHONY: all clean

all:
	cross build --target=x86_64-unknown-linux-musl --release
	docker build -t ghcr.io/backwardspy/casper7 .

clean:
	cargo clean
