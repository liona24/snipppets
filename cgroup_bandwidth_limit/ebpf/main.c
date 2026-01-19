#include <linux/types.h>
#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>

#define IFINDEX_LO 1

enum {
	DROP = 0,
	ALLOW = 1,
};

// section starts with ".maps" ==> BTF style map definition
struct {
	__uint(type, BPF_MAP_TYPE_ARRAY);
	__uint(max_entries, 2);
	__type(key, int);
	__type(value, __u64);
	__uint(map_flags, BPF_F_MMAPABLE);
} globals SEC(".maps");

#ifndef KIND
#define KIND "cgroup_skb/ingress"
#endif

const int BYTE_COUNT = 0;
const int HARD_QUOTA = 1;

SEC(KIND)
int bandwidth_limit(struct __sk_buff *skb) {
	// ignore loopback traffic
	if (skb->ifindex == IFINDEX_LO) {
		return ALLOW;
	}

	__u64* byte_count = bpf_map_lookup_elem(&globals, &BYTE_COUNT);
	if (byte_count == NULL) {
		return ALLOW;
	}

	__u64* el = bpf_map_lookup_elem(&globals, &HARD_QUOTA);
	__u64 hard_quota = el == NULL ? 0 : *el;

	if (hard_quota > 0 && *byte_count >= hard_quota) {
		return DROP;
	}

	__sync_fetch_and_add(byte_count, (__u64)skb->len);
	return ALLOW;
}

char _license[] SEC("license") = "MIT";
