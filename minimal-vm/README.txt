A minimal init ramfs with busybox.

Basic usage:

    ./create.sh

Run a VM with QEMU:

    qemu-system-x86_64 -kernel /boot/vmlinuz-XXX \
        -initrd init.cpio.gz \
        -nographic \
        -m 256 \
        -enable-kvm \
        -append "console=ttyS0"


Optionally, add networking support:
- Add to init:
    ifconfig lo 127.0.0.1 netmask 255.0.0.0 up

    # QEMU uses 10.0.2.0/24 by default. The builtin DHCP server issues IPs 
    # in the range 10.0.2.15-10.0.2.31
    # The host is reachable at 10.0.2.2
    ifconfig eth0 10.0.2.15 netmask 255.255.255.0

- Add to QEMU command line:
    -nic user,model=virtio-net-pci

