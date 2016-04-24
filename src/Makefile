ARCH ?= x86_64
TARGET ?= $(ARCH)-unknown-linux-gnu
KERNEL := build/kernel-$(ARCH).bin
ISO := build/os-$(ARCH).iso

RUST_BIN := kernel/target/$(TARGET)/debug/libkernel.a
LINKER_SCRIPT := arch/$(ARCH)/linker.ld
GRUB_CFG := arch/$(ARCH)/grub.cfg

ASM_SOURCES := $(wildcard arch/$(ARCH)/*.nasm)
ASM_OBJECTS := $(patsubst arch/$(ARCH)/%.nasm, build/arch/$(ARCH)/%.o, $(ASM_SOURCES))

all: $(KERNEL)

clean:
	@echo "[CLEAN] Cleaning build folder"
	@cd kernel && cargo clean && cd ..
	@rm -rf build
	
iso: $(KERNEL) $(GRUB_CFG)
	@echo "[GRUB ] $(KERNEL) -> $(ISO)"
	@mkdir -p build/iso/boot/grub
	@cp $(KERNEL) build/iso/boot/kernel.bin
	@cp $(GRUB_CFG) build/iso/boot/grub
	@grub-mkrescue -d /usr/lib/grub/i386-pc -o $(ISO) build/iso 2> /dev/null
	@rm -r build/iso
	
$(KERNEL): build $(ASM_OBJECTS) $(LINKER_SCRIPT) rust
	@echo "[LD   ] $(KERNEL)"
	@ld -n --gc-sections -T $(LINKER_SCRIPT) -o $(KERNEL) $(ASM_OBJECTS) $(RUST_BIN)
	
build:
	@mkdir -p build/arch/$(ARCH)
	
rust:
	@echo "[RUSTC] Compiling all rust code"
	@cd kernel && cargo rustc --target $(TARGET) -- -Z no-landing-pads && cd ..
	
build/arch/$(ARCH)/%.o: arch/$(ARCH)/%.nasm
	@echo "[NASM ] $< -> $@"
	@nasm -felf64 $< -o $@

run: iso
	@echo "[QEMU ] Launching QEMU..."
	@qemu-system-x86_64 -cdrom $(ISO) -s

debug: iso
	@echo "[QEMU ] Launching QEMU, waiting for debugger to connect on port 1234..."
	@qemu-system-x86_64 -cdrom $(ISO) -s -S
