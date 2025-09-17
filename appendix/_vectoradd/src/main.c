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

void double_vector_add_f64(double *a, double *b, double *result) {
    // 设置向量长度
    size_t gvl = __riscv_vsetvl_e64m1(ARRAY_SIZE);
    
    // 从 a 和 b 读取数据
    vfloat64m1_t v1 = __riscv_vle64_v_f64m1(a, gvl);
    vfloat64m1_t v2 = __riscv_vle64_v_f64m1(b, gvl);
    
    // 向量加法: v3 = v1 + v2
    vfloat64m1_t v3 = __riscv_vfadd_vv_f64m1(v1, v2, gvl);
    vfloat64m1_t v4 = __riscv_vfadd_vv_f64m1(v2, v3, gvl); // 再加一次作为示例
    
    // 将结果存储到 result
    __riscv_vse64_v_f64m1(result, v4, ARRAY_SIZE);
}

// 向量与标量依赖测试函数: 先进行标量计算，然后向量计算依赖标量结果
void vector_scalar_dependency_test(double *a, double *b, double *result) {
    // 1. 标量计算部分 - 计算数组的前几个元素的平均值作为偏移量
    double scalar_offset = 0.0;
    double scalar_multiplier = 1.0;
    
    // 计算前4个元素的平均值作为偏移量 (标量计算)
    for (int i = 0; i < 4; i++) {
        scalar_offset += a[i] + b[i];
    }
    scalar_offset = scalar_offset / 8.0; // 除以元素总数
    
    // 计算标量乘数 (依赖前面的标量计算)
    scalar_multiplier = 1.0 + (scalar_offset * 0.1); // 基于偏移量计算乘数
    
    // 2. 设置向量长度
    size_t gvl = __riscv_vsetvl_e64m1(ARRAY_SIZE);
    
    // 3. 向量计算部分 - 依赖标量计算的结果
    
    // 从 a 和 b 读取数据
    vfloat64m1_t v1 = __riscv_vle64_v_f64m1(a, gvl);
    vfloat64m1_t v2 = __riscv_vle64_v_f64m1(b, gvl);
    
    // 创建标量偏移量的向量版本 (标量到向量的依赖)
    vfloat64m1_t v_offset = __riscv_vfmv_v_f_f64m1(scalar_offset, gvl);
    vfloat64m1_t v_multiplier = __riscv_vfmv_v_f_f64m1(scalar_multiplier, gvl);
    
    // 向量加法: v3 = v1 + v2
    vfloat64m1_t v3 = __riscv_vfadd_vv_f64m1(v1, v2, gvl);
    
    // 加上标量偏移量 (向量计算依赖标量结果)
    vfloat64m1_t v4 = __riscv_vfadd_vv_f64m1(v3, v_offset, gvl);
    
    // 乘以标量乘数 (进一步的依赖关系)
    vfloat64m1_t v5 = __riscv_vfmul_vv_f64m1(v4, v_multiplier, gvl);
    
    // 将结果存储到 result
    __riscv_vse64_v_f64m1(result, v5, gvl);
}

// 标量加法操作用于验证结果
void scalar_add_f64(double *a, double *b, double *result) {
    for (int i = 0; i < ARRAY_SIZE; i++) {
        result[i] = a[i] + b[i];
    }
}

// 标量版本的依赖测试函数用于验证
void scalar_dependency_test(double *a, double *b, double *result) {
    // 1. 标量计算部分 - 与向量版本完全相同的逻辑
    double scalar_offset = 0.0;
    double scalar_multiplier = 1.0;
    
    // 计算前4个元素的平均值作为偏移量
    for (int i = 0; i < 4; i++) {
        scalar_offset += a[i] + b[i];
    }
    scalar_offset = scalar_offset / 8.0;
    
    // 计算标量乘数
    scalar_multiplier = 1.0 + (scalar_offset * 0.1);
    
    // 2. 对每个元素执行相同的操作
    for (int i = 0; i < ARRAY_SIZE; i++) {
        double temp = a[i] + b[i];           // 对应向量加法
        temp = temp + scalar_offset;         // 对应加偏移量
        result[i] = temp * scalar_multiplier; // 对应乘乘数
    }
}

int main() {
    srand(time(NULL));
    
    // 创建对齐的数组
    double a[ARRAY_SIZE] __attribute__((aligned(16)));
    double b[ARRAY_SIZE] __attribute__((aligned(16)));
    double vector_result[ARRAY_SIZE] __attribute__((aligned(16)));
    double scalar_result[ARRAY_SIZE] __attribute__((aligned(16)));
    double dependency_vector_result[ARRAY_SIZE] __attribute__((aligned(16)));
    double dependency_scalar_result[ARRAY_SIZE] __attribute__((aligned(16)));

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

    // 执行基本向量加法
    printf("=== Basic Vector Addition Test ===\n");
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

    // 验证基本结果正确性
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
        printf("\n✓ Basic vector addition test PASSED.\n\n");
    } else {
        printf("\n✗ Basic vector addition test FAILED.\n\n");
    }

    // 执行Vector-Scalar依赖测试
    printf("=== Vector-Scalar Dependency Test ===\n");
    printf("Testing vector operations that depend on scalar calculations...\n");
    
    vector_scalar_dependency_test(a, b, dependency_vector_result);
    scalar_dependency_test(a, b, dependency_scalar_result);

    printf("Vector-Scalar dependency results:\n");
    for (int i = 0; i < ARRAY_SIZE; i++) {
        printf("dependency_result[%d] = %8.2f\n", i, dependency_vector_result[i]);
    }
    printf("\n");

    // 验证依赖测试结果正确性
    printf("Dependency verification (comparing with scalar implementation):\n");
    int dependency_correct = 1;
    for (int i = 0; i < ARRAY_SIZE; i++) {
        double diff = dependency_vector_result[i] - dependency_scalar_result[i];
        if (diff < -0.0001 || diff > 0.0001) {  // 允许小的浮点误差
            printf("MISMATCH at index %d: vector=%8.2f, scalar=%8.2f\n", 
                   i, dependency_vector_result[i], dependency_scalar_result[i]);
            dependency_correct = 0;
        } else {
            printf("OK at index %d: %8.2f\n", i, dependency_vector_result[i]);
        }
    }

    if (dependency_correct) {
        printf("\n✓ Vector-Scalar dependency test PASSED.\n");
    } else {
        printf("\n✗ Vector-Scalar dependency test FAILED.\n");
    }

    printf("\n=== Summary ===\n");
    printf("Basic vector addition: %s\n", all_correct ? "PASSED" : "FAILED");
    printf("Vector-scalar dependency: %s\n", dependency_correct ? "PASSED" : "FAILED");

    return (all_correct && dependency_correct) ? 0 : 1;
}
