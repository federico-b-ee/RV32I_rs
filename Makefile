
# Compiler and flags
RUSTC = rustc
TARGET = riscv32i-unknown-none-elf
RUSTFLAGS = --target=$(TARGET)

# Output directory and binary name
OUT_DIR = target
BIN_NAME = riscv_asm

# Build target


build_elf: clean 
	cd example/riscv_asm && \
	cargo build --bin $(BIN_NAME) && \
	cp $(OUT_DIR)/$(TARGET)/debug/$(BIN_NAME) ../$(BIN_NAME).elf && \
	mkdir -p ../binaries && \
	rust-objcopy -O binary --only-section=.text ../$(BIN_NAME).elf ../binaries/$(BIN_NAME).bin && \
	hexdump -ve '1/4 "%08x\n"' ../binaries/$(BIN_NAME).bin > ../binaries/$(BIN_NAME).prog && \
	rust-objcopy -O binary --only-section=.data ../$(BIN_NAME).elf ../binaries/$(BIN_NAME).bin && \
	hexdump -ve '1/4 "%08x\n"' ../binaries/$(BIN_NAME).bin > ../binaries/$(BIN_NAME).mem && \
	rm -r ../binaries/$(BIN_NAME).bin

run_demo: build_elf
	cargo run

clean_build_elf:
	rm -f $(BIN_NAME)/$(BIN_NAME).elf
	rm -rf $(BIN_NAME)/$(OUT_DIR)


