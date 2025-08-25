#include <stdint.h>
#include <riscv_vector.h>
#include <riscv_vector_v0p10.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define ARRAY_SIZE 16

// 随机生成 double 类型数组
void generate_random_data(double *arr, size_t size) {
    for (size_t i = 0; i < size; i++) {
        arr[i] = (rand() % 100) + (rand() % 100) * 0.01; // 生成 0-99.99 范围的数
    }
}

// 向量加法操作: result = a + b
void vector_add_f64(double *a, double *b, double *result) {
    // 设置向量长度
    size_t gvl = __riscv_vsetvl_e64m1(ARRAY_SIZE);
    
    // 从 a 和 b 读取数据
    vfloat64m1_t v1 = __riscv_vle64_v_f64m1(a, gvl);
    vfloat64m1_t v2 = __riscv_vle64_v_f64m1(b, gvl);
    
    // 向量加法: v3 = v1 + v2
    vfloat64m1_t v3 = __riscv_vfadd_vv_f64m1(v1, v2, gvl);
    
    // 将结果存储到 result
    __riscv_vse64_v_f64m1(result, v3, ARRAY_SIZE);
}

// 标量加法操作用于验证结果
void scalar_add_f64(double *a, double *b, double *result) {
    for (int i = 0; i < ARRAY_SIZE; i++) {
        result[i] = a[i] + b[i];
    }
}

int main() {
    srand(time(NULL));
    
    // 创建对齐的数组
    double a[ARRAY_SIZE] __attribute__((aligned(16)));
    double b[ARRAY_SIZE] __attribute__((aligned(16)));
    double vector_result[ARRAY_SIZE] __attribute__((aligned(16)));
    double scalar_result[ARRAY_SIZE] __attribute__((aligned(16)));

    // 生成随机数据
    generate_random_data(a, ARRAY_SIZE);
    generate_random_data(b, ARRAY_SIZE);

    printf("=== RISC-V Vector Add Test ===\n");
    printf("Array size: %d\n\n", ARRAY_SIZE);

    // 输出输入数据
    printf("Input data:\n");
    for (int i = 0; i < ARRAY_SIZE; i++) {
        printf("a[%d] = %8.2f, b[%d] = %8.2f\n", i, a[i], i, b[i]);
    }
    printf("\n");

    // 执行向量加法
    printf("Executing vector addition...\n");
    vector_add_f64(a, b, vector_result);

    // 执行标量加法用于验证
    scalar_add_f64(a, b, scalar_result);

    // 打印向量加法结果
    printf("Vector addition results:\n");
    for (int i = 0; i < ARRAY_SIZE; i++) {
        printf("vector_result[%d] = %8.2f\n", i, vector_result[i]);
    }
    printf("\n");

    // 验证结果正确性
    printf("Verification (comparing with scalar results):\n");
    int all_correct = 1;
    for (int i = 0; i < ARRAY_SIZE; i++) {
        double diff = vector_result[i] - scalar_result[i];
        if (diff < -0.0001 || diff > 0.0001) {  // 允许小的浮点误差
            printf("MISMATCH at index %d: vector=%8.2f, scalar=%8.2f\n", 
                   i, vector_result[i], scalar_result[i]);
            all_correct = 0;
        } else {
            printf("OK at index %d: %8.2f\n", i, vector_result[i]);
        }
    }

    if (all_correct) {
        printf("\n✓ All results match! Vector addition test PASSED.\n");
    } else {
        printf("\n✗ Some results mismatch! Vector addition test FAILED.\n");
    }

    return all_correct ? 0 : 1;
}
