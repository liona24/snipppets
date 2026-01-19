.PHONY: all
all:
	$(MAKE) -C ebpf/ all
	cargo build --release

.PHONY: clean
clean:
	$(MAKE) -C ebpf/ clean
	cargo clean
