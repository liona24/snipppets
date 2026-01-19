# -g is needed for BTF info
CFLAGS	:= -g -O2 -Wall -Werror -target bpf -c $(CFLAGS)
SRC	:= main.c

all: ebpf_ingress.o ebpf_egress.o $(SRC)

ebpf_ingress.o: $(SRC)
	clang $(CFLAGS) -DKIND="\"cgroup_skb/ingress\"" -o $@ $^

ebpf_egress.o: $(SRC)
	clang $(CFLAGS) -DKIND="\"cgroup_skb/egress\"" -o $@ $^

.PHONY: clean
clean:
	rm -f ebpf_ingress.o
	rm -f ebpf_egress.o
