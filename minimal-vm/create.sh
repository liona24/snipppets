#!/usr/bin/bash

set -xe

if [[ ! -f busybox ]]; then
    wget https://www.busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox
    chmod +x busybox
fi

ROOT=img

if [[ -d $ROOT ]]; then
    sudo rm -r $ROOT
fi

mkdir -p $ROOT/bin $ROOT/lib $ROOT/lib64

mkdir $ROOT/etc
echo "root:x:0:0:root:/root:/bin/sh" > $ROOT/etc/passwd

mkdir $ROOT/proc $ROOT/tmp $ROOT/sys $ROOT/dev

cp init $ROOT/init
cp busybox $ROOT/bin
sudo chroot $ROOT /bin/busybox --install -s /bin

sudo chown -R root:root $ROOT
sudo chmod +s $ROOT/bin/busybox

pushd $ROOT
find . -print0 | cpio --null -ov --format=newc  2> /dev/null | gzip -9 > "../init.cpio.gz" 2> /dev/null
popd
