FROM scratch
COPY target/x86_64-unknown-linux-musl/release/casper7 .
CMD ["./casper7"]
