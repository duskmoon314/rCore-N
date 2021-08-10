TARGET := riscv64imac-unknown-none-elf
MODE := debug
APP_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
APPS := $(wildcard $(APP_DIR)/*.rs)
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))
BINS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%.bin, $(APPS))

OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

elf: $(APPS)
	@cargo build

binary: elf
	$(foreach elf, $(ELFS), $(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.bin, $(elf));)
	$(foreach elf, $(ELFS), $(OBJDUMP) -D -S $(elf) > $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.asm, $(elf));)

build: binary

clean:
	@cargo clean

.PHONY: elf binary build clean
