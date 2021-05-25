#!/bin/bash

if [ $# -ne 2 ]
then
    echo "ERROR: Invalid arguments. Only 2 arg 1 -> filesystem_name 2 -> mount point"
    exit 1
fi

mount_point="$2"

# Crea el sistema de fitxers.
dd if=/dev/zero of="$1" bs=1024 count=102400
mkfs.fat -F 16 "$1"
chmod 766 "$1"

# Montal
umount "$mount_point"
mount "$1" "$mount_point"

# Crea un fitxer grandet
for i in {0..9999}; do echo "$i" >> "$mount_point"/abcd;done
for i in {0..9999}; do echo "$i" >> "$mount_point"/fghi.txt;done

# Crea una carpeta amb molts fitxers.
mkdir "$mount_point"/greatdir
touch "$mount_point"/greatdir/file{000..999}.txt