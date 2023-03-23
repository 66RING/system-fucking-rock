TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF = target/$(TARGET)/$(MODE)/os
KERNEL_BIN = $(KERNEL_ELF).bin
BOOTLOADER = ../bootloader/rustsbi-qemu.bin

# 指定系统二进制代码位置, qemu将pc指针置位
KERNEL_ENTRY_PA = 0x80200000

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# 镜像位置
FS_IMG := ../user/target/$(TARGET)/release/fs.img

ifeq ($(MODE), release)
	MODE_ARG := --release
endif


build: env kernel $(KERNEL_BIN) fs-img

# 要裁剪掉metadata加载时pc才是正确的位置
# 裁剪后生成到.bin中($@表示target)
$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@


# 检测rust环境, 自动补完
env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils --vers =0.3.3
	rustup component add rust-src
	rustup component add llvm-tools-preview


# 构建内核和用户程序
kernel:
	@cd ../user && make build
	@cargo build $(MODE_ARG)


run: build
	qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios $(BOOTLOADER) \
            -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) \
			-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
			-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

gdb: build
	qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios $(BOOTLOADER) \
            -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) -S -s \
			-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
			-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

# 编译镜像
fs-img: $(APPS)
	@cd ../user && make build TEST=$(TEST)
	@rm -f $(FS_IMG)
	@cd ../easy-fs-fuse && cargo run --release -- -s ../user/src/bin/ -t ../user/target/riscv64gc-unknown-none-elf/release/


clean:
	cargo clean
	@cd ../user && make clean

.PHONY: build clean run
