#include <stdint.h>
#include <riscv_vector.h>
#include <riscv_vector_v0p10.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <math.h>

#define MATRIX_SIZE 4  // 4x4 矩阵
#define TOTAL_ELEMENTS (MATRIX_SIZE * MATRIX_SIZE)

// 随机生成 double 类型矩阵
void generate_random_matrix(double *matrix, size_t size) {
    for (size_t i = 0; i < size; i++) {
        matrix[i] = (rand() % 10) + (rand() % 100) * 0.01; // 生成 0-9.99 范围的数
    }
}

// 打印矩阵
void print_matrix(const char *name, double *matrix, int rows, int cols) {
    printf("%s:\n", name);
    for (int i = 0; i < rows; i++) {
        for (int j = 0; j < cols; j++) {
            printf("%8.2f ", matrix[i * cols + j]);
        }
        printf("\n");
    }
    printf("\n");
}

// 矩阵元素加法 (使用向量指令): C = A + B
void matrix_add_vector(double *a, double *b, double *c) {
    // 设置向量长度
    size_t gvl = __riscv_vsetvl_e64m1(TOTAL_ELEMENTS);
    
    for (size_t i = 0; i < TOTAL_ELEMENTS; i += gvl) {
        // 更新向量长度，处理剩余元素
        gvl = __riscv_vsetvl_e64m1(TOTAL_ELEMENTS - i);
        
        // 从矩阵 A 和 B 读取数据
        vfloat64m1_t v1 = __riscv_vle64_v_f64m1(&a[i], gvl);
        vfloat64m1_t v2 = __riscv_vle64_v_f64m1(&b[i], gvl);
        
        // 向量加法: v3 = v1 + v2
        vfloat64m1_t v3 = __riscv_vfadd_vv_f64m1(v1, v2, gvl);
        
        // 将结果存储到矩阵 C
        __riscv_vse64_v_f64m1(&c[i], v3, gvl);
    }
}

void matrix_sub_vector(double *a, double *b, double *c) {
    // 设置向量长度
    size_t gvl = __riscv_vsetvl_e64m1(TOTAL_ELEMENTS);
    
    for (size_t i = 0; i < TOTAL_ELEMENTS; i += gvl) {
        // 更新向量长度，处理剩余元素
        gvl = __riscv_vsetvl_e64m1(TOTAL_ELEMENTS - i);
        
        // 从矩阵 A 和 B 读取数据
        vfloat64m1_t v1 = __riscv_vle64_v_f64m1(&a[i], gvl);
        vfloat64m1_t v2 = __riscv_vle64_v_f64m1(&b[i], gvl);
        
        // 向量加法: v3 = v1 + v2
        vfloat64m1_t v3 = __riscv_vfsub_vv_f64m1(v1, v2, gvl);
        
        // 将结果存储到矩阵 C
        __riscv_vse64_v_f64m1(&c[i], v3, gvl);
    }
}

// 矩阵元素乘法 (使用向量指令): C = A * B (element-wise)
void matrix_mul_vector(double *a, double *b, double *c) {
    // 设置向量长度
    size_t gvl = __riscv_vsetvl_e64m1(TOTAL_ELEMENTS);
    
    for (size_t i = 0; i < TOTAL_ELEMENTS; i += gvl) {
        // 更新向量长度，处理剩余元素
        gvl = __riscv_vsetvl_e64m1(TOTAL_ELEMENTS - i);
        
        // 从矩阵 A 和 B 读取数据
        vfloat64m1_t v1 = __riscv_vle64_v_f64m1(&a[i], gvl);
        vfloat64m1_t v2 = __riscv_vle64_v_f64m1(&b[i], gvl);
        
        // 向量乘法: v3 = v1 * v2
        vfloat64m1_t v3 = __riscv_vfmul_vv_f64m1(v1, v2, gvl);
        
        // 将结果存储到矩阵 C
        __riscv_vse64_v_f64m1(&c[i], v3, gvl);
    }
}

// 混合运算 (使用向量指令): D = (A + B) * C
void matrix_fused_operation(double *a, double *b, double *c, double *d) {
    // 设置向量长度
    size_t gvl = __riscv_vsetvl_e64m1(TOTAL_ELEMENTS);
    
    for (size_t i = 0; i < TOTAL_ELEMENTS; i += gvl) {
        // 更新向量长度，处理剩余元素
        gvl = __riscv_vsetvl_e64m1(TOTAL_ELEMENTS - i);
        
        // 从矩阵 A, B, C 读取数据
        vfloat64m1_t va = __riscv_vle64_v_f64m1(&a[i], gvl);
        vfloat64m1_t vb = __riscv_vle64_v_f64m1(&b[i], gvl);
        vfloat64m1_t vc = __riscv_vle64_v_f64m1(&c[i], gvl);
        
        // 第一步: 向量加法 temp = va + vb
        vfloat64m1_t vtemp = __riscv_vfadd_vv_f64m1(va, vb, gvl);
        
        // 第二步: 向量乘法 vd = temp * vc
        vfloat64m1_t vd = __riscv_vfmul_vv_f64m1(vtemp, vc, gvl);
        
        // 将结果存储到矩阵 D
        __riscv_vse64_v_f64m1(&d[i], vd, gvl);
    }
}

// 标量版本，用于验证结果
void matrix_add_scalar(double *a, double *b, double *c) {
    for (int i = 0; i < TOTAL_ELEMENTS; i++) {
        c[i] = a[i] + b[i];
    }
}

void matrix_mul_scalar(double *a, double *b, double *c) {
    for (int i = 0; i < TOTAL_ELEMENTS; i++) {
        c[i] = a[i] * b[i];
    }
}

void matrix_fused_scalar(double *a, double *b, double *c, double *d) {
    for (int i = 0; i < TOTAL_ELEMENTS; i++) {
        d[i] = (a[i] + b[i]) * c[i];
    }
}

// 比较两个矩阵是否相等
int compare_matrices(double *m1, double *m2, const char *test_name) {
    const double EPSILON = 1e-10;
    int errors = 0;
    
    printf("Verifying %s...\n", test_name);
    for (int i = 0; i < TOTAL_ELEMENTS; i++) {
        if (fabs(m1[i] - m2[i]) > EPSILON) {
            printf("  ERROR at position %d: vector=%8.2f, scalar=%8.2f\n", i, m1[i], m2[i]);
            errors++;
        }
    }
    
    if (errors == 0) {
        printf("  ✓ %s PASSED - All results match!\n", test_name);
    } else {
        printf("  ✗ %s FAILED - %d mismatches found!\n", test_name, errors);
    }
    printf("\n");
    return errors;
}

int main() {
    srand(time(NULL));
    
    // 创建对齐的矩阵
    double matrix_a[TOTAL_ELEMENTS] __attribute__((aligned(16)));
    double matrix_b[TOTAL_ELEMENTS] __attribute__((aligned(16)));
    double matrix_c[TOTAL_ELEMENTS] __attribute__((aligned(16)));
    double matrix_d[TOTAL_ELEMENTS] __attribute__((aligned(16)));
    
    double vector_result_add[TOTAL_ELEMENTS] __attribute__((aligned(16)));
    double vector_result_mul[TOTAL_ELEMENTS] __attribute__((aligned(16)));
    double vector_result_fused[TOTAL_ELEMENTS] __attribute__((aligned(16)));
    
    double scalar_result_add[TOTAL_ELEMENTS] __attribute__((aligned(16)));
    double scalar_result_mul[TOTAL_ELEMENTS] __attribute__((aligned(16)));
    double scalar_result_fused[TOTAL_ELEMENTS] __attribute__((aligned(16)));

    // 生成随机矩阵数据
    generate_random_matrix(matrix_a, TOTAL_ELEMENTS);
    generate_random_matrix(matrix_b, TOTAL_ELEMENTS);
    generate_random_matrix(matrix_c, TOTAL_ELEMENTS);

    printf("=== RISC-V Vector Matrix Operations Test ===\n");
    printf("Matrix size: %dx%d (%d elements)\n\n", MATRIX_SIZE, MATRIX_SIZE, TOTAL_ELEMENTS);

    // 打印输入矩阵
    print_matrix("Matrix A", matrix_a, MATRIX_SIZE, MATRIX_SIZE);
    print_matrix("Matrix B", matrix_b, MATRIX_SIZE, MATRIX_SIZE);
    print_matrix("Matrix C", matrix_c, MATRIX_SIZE, MATRIX_SIZE);

    // 测试1: 矩阵加法 A + B
    printf("=== Test 1: Matrix Addition (A + B) ===\n");
    matrix_add_vector(matrix_a, matrix_b, vector_result_add);
    matrix_add_scalar(matrix_a, matrix_b, scalar_result_add);
    print_matrix("Vector Result (A + B)", vector_result_add, MATRIX_SIZE, MATRIX_SIZE);
    int errors1 = compare_matrices(vector_result_add, scalar_result_add, "Matrix Addition");

    // 测试2: 矩阵逐元素乘法 A * B
    printf("=== Test 2: Element-wise Multiplication (A * B) ===\n");
    matrix_mul_vector(matrix_a, matrix_b, vector_result_mul);
    matrix_mul_scalar(matrix_a, matrix_b, scalar_result_mul);
    print_matrix("Vector Result (A * B)", vector_result_mul, MATRIX_SIZE, MATRIX_SIZE);
    int errors2 = compare_matrices(vector_result_mul, scalar_result_mul, "Element-wise Multiplication");

    // 测试3: 混合运算 (A + B) * C
    printf("=== Test 3: Fused Operation ((A + B) * C) ===\n");
    matrix_fused_operation(matrix_a, matrix_b, matrix_c, vector_result_fused);
    matrix_fused_scalar(matrix_a, matrix_b, matrix_c, scalar_result_fused);
    print_matrix("Vector Result ((A + B) * C)", vector_result_fused, MATRIX_SIZE, MATRIX_SIZE);
    int errors3 = compare_matrices(vector_result_fused, scalar_result_fused, "Fused Operation");

    // 总结测试结果
    printf("=== Test Summary ===\n");
    int total_errors = errors1 + errors2 + errors3;
    if (total_errors == 0) {
        printf("🎉 ALL TESTS PASSED! Vector matrix operations working correctly.\n");
    } else {
        printf("❌ %d TESTS FAILED! Total errors: %d\n", (errors1 > 0) + (errors2 > 0) + (errors3 > 0), total_errors);
    }

    return total_errors;
}
