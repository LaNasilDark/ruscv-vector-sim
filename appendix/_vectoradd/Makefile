#makefile for vector add test
# RISCV=~/plct-gem5/Xuantie-900-llvm-elf-newlib-x86_64-V2.0.1 # Change This to your riscv path
RISCV=~/riscv-toolchain/xuantie/xuantie-llvm-elf

start:
	rm bin/*.*

vector-old:
	echo "vector add test"
	${RISCV}/bin/clang -march=rv64gcv -DUSE_RISCV_VECTOR -O1 -fno-vectorize -funroll-loops --static -o bin/vectoradd src/main.c -lm5
	${RISCV}/bin/llvm-objdump --mattr=+m,+f,+d,+a,+c,+v -S bin/vectoradd > bin/vectoradd.dump

vector:
	echo "vector add test"
	mkdir -p bin
	${RISCV}/bin/clang -march=rv64gcv -DUSE_RISCV_VECTOR -O2 -fno-vectorize -funroll-loops -I../../common/ --static -o bin/vectoradd src/main.c
	${RISCV}/bin/llvm-objdump --mattr=+m,+f,+d,+a,+c,+v -S bin/vectoradd > bin/vectoradd.dump

clean:
	rm -f bin/*

.PHONY: start vector-old vector clean
