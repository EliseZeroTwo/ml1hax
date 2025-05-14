rm -f payload.bin && cargo clean && cargo build-dump-instruction-addr-bin && mv payload.bin ../../fastbootrs/src/payload.bin
