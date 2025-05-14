rm -f payload.bin && cargo clean && cargo build-dump-memory-bin && mv payload.bin ../../fastbootrs/src/payload.bin
