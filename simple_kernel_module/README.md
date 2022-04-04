Simple Kernel Module
====================

## Build it

```bash
make
```

## Load it

```bash
insmod simpledev.ko
major=$(awk '$2 == "simpledev" { print $1 }' /proc/devices)
minor=0
mknod /dev/simpledev c $major $minor
chmod 666 /dev/simpledev
```

## Use it

```c
#include <stdio.h>
#include <sys/types.h>
#include <string.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/ioctl.h>

struct request {
    char msg[10];
};

#define CMD_GIVE _IOW(0xF00BA6, 1, struct request)
#define CMD_TAKE _IOR(0xF00BA6, 2, struct request)

int main() {
    printf("[*] opening device ..\n");
    int fd = open("/dev/simpledev", O_RDWR);

    if (fd < 0) {
        perror("open(\"/dev/simpledev\")");
        return fd;
    }

    printf("[+] opened /dev/simpledev\n");

    printf("[*] testing read ..\n");
    char buf[30] = {0};
    if (read(fd, buf, sizeof(buf)) < 0) {
        perror("read()");
        return -1;
    }
    printf("[+] got \"%s\"\n", buf);

    printf("[*] testing write ..\n");
    char msg[] = "beep boop";
    if (write(fd, msg, sizeof(msg)) < 0) {
        perror("write()");
        return -1;
    }
    printf("[+] check dmesg :)\n");

    printf("[*] testing ioctl give ..\n");
    struct request req = {
        .msg = "ioctltest"
    };
    if (ioctl(fd, CMD_GIVE, &req)) {
        perror("ioctl(fd, CMD_GIVE, &req)");
        return -1;
    }
    printf("[*] testing ioctl take ..\n");
    memset(req.msg, 0, sizeof(req.msg));
    if (ioctl(fd, CMD_TAKE, &req)) {
        perror("ioctl(fd, CMD_TAKE, &req)");
        return -1;
    }
    printf("[+] got \"%s\"\n", req.msg);

    return 0;
}

```

## Unload it

```
rmmod simpledev
# or
modprobe -r simpledev

rm -f /dev/simpledev
```
