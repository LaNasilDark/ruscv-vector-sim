# Makefile for vector-scalar dependency test
# RISCV=~/plct-gem5/Xuantie-900-llvm-elf-newlib-x86_64-V2.0.1 # Change This to your riscv path
RISCV=~/riscv-toolchain/xuantie/xuantie-llvm-elf

# 默认目标
all: vector

start:
	rm -f bin/*.*

# 编译向量标量依赖测试
vector:
	echo "Building vector-scalar dependency test"
	mkdir -p bin
	${RISCV}/bin/clang -march=rv64gcv -DUSE_RISCV_VECTOR -O2 -fno-vectorize -funroll-loops -I../../common/ --static -o bin/vector_scalar_dep src/main.c
	${RISCV}/bin/llvm-objdump --mattr=+m,+f,+d,+a,+c,+v -S bin/vector_scalar_dep > bin/vector_scalar_dep.dump

# 清理编译产物
clean:
	rm -f bin/*

# 运行测试（如果支持的话）
run: vector
	echo "Running vector-scalar dependency test..."
	./bin/vector_scalar_dep

# 查看反汇编
dump: vector
	echo "Displaying assembly dump..."
	cat bin/vector_scalar_dep.dump

# 显示帮助信息
help:
	@echo "Available targets:"
	@echo "  all      - Build the vector-scalar dependency test (default)"
	@echo "  vector   - Build the vector-scalar dependency test"
	@echo "  clean    - Remove all build artifacts"
	@echo "  run      - Build and run the test"
	@echo "  dump     - Show the assembly dump"
	@echo "  start    - Clean build artifacts"
	@echo "  help     - Show this help message"

.PHONY: all start vector clean run dump help