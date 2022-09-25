MODE := debug
KERNEL_BIN = target/riscv64gc-unknown-none-elf/$(MODE)/os
BOOTLOADER = ../bootloader/rustsbi-qemu.bin

# 指定系统二进制代码位置, qemu将pc指针置位
KERNEL_ENTRY_PA = 0x80200000

build:
	cargo build

run: build
	qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios $(BOOTLOADER) \
            -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)

clean:
	cargo clean

.PHONY: build clean
