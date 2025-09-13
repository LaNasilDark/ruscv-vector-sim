#makefile
RISCV=~/riscv-toolchain/xuantie/xuantie-llvm-elf
start:
	rm -f bin/*.*

serial:
	echo "serial"
	${RISCV}/bin/clang -march=rv64gcv -O2 -fno-vectorize -o bin/matmul_serial.exe src/matmul.c src/main.c
	${RISCV}/bin/llvm-objdump  --mattr=+m,+f,+d,+a,+c -ds  bin/matmul_serial.exe > bin/matmul_serial.dump

vector:
	echo "vector"
	${RISCV}/bin/clang -march=rv64gcv -DUSE_RISCV_VECTOR -O3 -fno-vectorize -funroll-loops -I../../include/ -L../../util/m5/build/riscv/out  --static -o bin/matmul_vector.exe src/matmul.c src/main.c
	${RISCV}/bin/llvm-objdump  --mattr=+m,+f,+d,+a,+c,+v -S  bin/matmul_vector.exe > bin/matmul_vector.dump

clean:
	rm -f bin/*.dump; \
	rm -f bin/*.exe;
