#makefile
RISCV=~/plct-gem5/Xuantie-900-llvm-elf-newlib-x86_64-V2.0.1

start:
	rm -f bin/*.*


vector:
	echo "vector"
	${RISCV}/bin/clang -march=rv64gcv -DUSE_RISCV_VECTOR -O3 -fno-vectorize -funroll-loops --static -o bin/conflict_port.exe src/main.c
	${RISCV}/bin/llvm-objdump  --mattr=+m,+f,+d,+a,+c,+v -S  bin/conflict_port.exe > bin/conflict_port.dump

clean:
	rm -f bin/*.dump; \
	rm -f bin/*.exe;