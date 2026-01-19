# Bandwidth limiting kernel hooks and userspace helpers

This project installs a simple cgroup packet hook that counts ingress/egress bytes
and drops packets when the configured limit is exceeded.

To support this a simple userspace helper is provided which displays statistics
and resets counters periodically.

## Prerequisites

Install clang and `libbpf` for your platform.


## Build

```
make
```

## Run

(requires `/proc/sys/kernel/unprivileged_bpf_disabled == 0` or `CAP_BPF` and access to the cgroup)

```
./target/release/bandwidth-limit --help
Usage: bandwidth-limit --cgroup <CGROUP> --quota <QUOTA> --quota-period <QUOTA_PERIOD> --sample-interval <SAMPLE_INTERVAL>

Options:
  -c, --cgroup <CGROUP>
          Cgroup to attach to (absolute path)
  -q, --quota <QUOTA>
          Allowed quota for ingress / egress each per quota period. Number of bytes
  -q, --quota-period <QUOTA_PERIOD>
          Quota period given in number of seconds
  -s, --sample-interval <SAMPLE_INTERVAL>
          Metrics collection sample interval in number of seconds
  -h, --help
          Print help
```

Example, limit to 10MiB per 10 sec. Report every second:
```
RUST_LOG=info ./target/release/bandwidth-limit --cgroup /sys/fs/cgroup/foo --quota 10485760 --quota-period 10 --sample-interval 1
```
