#include <stdint.h>
#include <riscv_vector.h>
#include <riscv_vector_v0p10.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define ARRAY_SIZE 16

// 测试用例1: 向量指令依赖标量计算的结果
// 场景：标量寄存器计算 -> 向量指令使用该标量结果
void test_vector_depends_on_scalar() {
    printf("=== Test 1: Vector instruction depends on scalar computation ===\n");
    
    double a[ARRAY_SIZE] __attribute__((aligned(16)));
    double result[ARRAY_SIZE] __attribute__((aligned(16)));
    
    // 初始化数组
    for (int i = 0; i < ARRAY_SIZE; i++) {
        a[i] = i + 1.0;
    }
    
    // 标量计算：计算一个缩放因子
    double base_factor = 2.5;
    double increment = 0.1;
    double scale_factor = base_factor + increment;  // 标量依赖：scale_factor依赖于base_factor和increment
    
    printf("Base factor: %.2f, Increment: %.2f, Scale factor: %.2f\n", 
           base_factor, increment, scale_factor);
    
    // 向量操作：使用标量计算的结果作为向量广播的数值
    size_t gvl = __riscv_vsetvl_e64m1(ARRAY_SIZE);
    
    // 加载向量数据
    vfloat64m1_t va = __riscv_vle64_v_f64m1(a, gvl);
    
    // 向量标量乘法：向量指令依赖于前面计算的标量值scale_factor
    vfloat64m1_t vresult = __riscv_vfmul_vf_f64m1(va, scale_factor, gvl);
    
    // 存储结果
    __riscv_vse64_v_f64m1(result, vresult, gvl);
    
    // 验证结果
    printf("Results (original * scale_factor):\n");
    int all_correct = 1;
    for (int i = 0; i < ARRAY_SIZE; i++) {
        double expected = a[i] * scale_factor;
        double diff = result[i] - expected;
        printf("a[%d] = %.2f, result[%d] = %.2f, expected = %.2f", 
               i, a[i], i, result[i], expected);
        if (diff < -0.0001 || diff > 0.0001) {
            printf(" [FAIL]\n");
            all_correct = 0;
        } else {
            printf(" [OK]\n");
        }
    }
    
    if (all_correct) {
        printf("✓ Test 1 PASSED\n\n");
    } else {
        printf("✗ Test 1 FAILED\n\n");
    }
}

// 测试用例2: 标量地址计算 -> 向量内存访问
// 场景：标量寄存器计算地址偏移 -> 向量加载指令使用该地址
void test_vector_memory_depends_on_scalar_address() {
    printf("=== Test 2: Vector memory access depends on scalar address computation ===\n");
    
    double data[ARRAY_SIZE * 2] __attribute__((aligned(16)));  // 双倍大小的数组
    double result[ARRAY_SIZE] __attribute__((aligned(16)));
    
    // 初始化数据数组
    for (int i = 0; i < ARRAY_SIZE * 2; i++) {
        data[i] = (i + 1) * 1.5;
    }
    
    // 标量地址计算：计算偏移量
    int base_offset = 4;
    int additional_offset = 2;
    int total_offset = base_offset + additional_offset;  // 标量依赖：total_offset依赖于两个标量相加
    
    printf("Base offset: %d, Additional offset: %d, Total offset: %d\n", 
           base_offset, additional_offset, total_offset);
    
    // 向量内存访问：使用标量计算的偏移地址
    size_t gvl = __riscv_vsetvl_e64m1(ARRAY_SIZE);
    
    // 从计算出的偏移地址加载数据：向量加载指令依赖于标量地址计算
    vfloat64m1_t vdata = __riscv_vle64_v_f64m1(&data[total_offset], gvl);
    
    // 简单的向量操作
    vfloat64m1_t vresult = __riscv_vfadd_vf_f64m1(vdata, 10.0, gvl);
    
    // 存储结果
    __riscv_vse64_v_f64m1(result, vresult, gvl);
    
    // 验证结果
    printf("Results (data[offset+i] + 10.0):\n");
    int all_correct = 1;
    for (int i = 0; i < ARRAY_SIZE; i++) {
        double expected = data[total_offset + i] + 10.0;
        double diff = result[i] - expected;
        printf("data[%d] = %.2f, result[%d] = %.2f, expected = %.2f", 
               total_offset + i, data[total_offset + i], i, result[i], expected);
        if (diff < -0.0001 || diff > 0.0001) {
            printf(" [FAIL]\n");
            all_correct = 0;
        } else {
            printf(" [OK]\n");
        }
    }
    
    if (all_correct) {
        printf("✓ Test 2 PASSED\n\n");
    } else {
        printf("✗ Test 2 FAILED\n\n");
    }
}

// 测试用例3: 混合标量-向量依赖链
// 场景：标量计算 -> 向量操作 -> 标量归约 -> 向量操作
void test_mixed_scalar_vector_dependency_chain() {
    printf("=== Test 3: Mixed scalar-vector dependency chain ===\n");
    
    double a[ARRAY_SIZE] __attribute__((aligned(16)));
    double b[ARRAY_SIZE] __attribute__((aligned(16)));
    double result[ARRAY_SIZE] __attribute__((aligned(16)));
    
    // 初始化数组
    for (int i = 0; i < ARRAY_SIZE; i++) {
        a[i] = i + 1.0;
        b[i] = (i + 1) * 0.5;
    }
    
    // 步骤1: 标量计算初始参数
    double param1 = 3.0;
    double param2 = 2.0;
    double multiplier = param1 * param2;  // 标量依赖
    
    printf("Initial scalar computation: %.2f * %.2f = %.2f\n", param1, param2, multiplier);
    
    // 步骤2: 向量操作1 - 依赖于标量计算的结果
    size_t gvl = __riscv_vsetvl_e64m1(ARRAY_SIZE);
    vfloat64m1_t va = __riscv_vle64_v_f64m1(a, gvl);
    vfloat64m1_t vb = __riscv_vle64_v_f64m1(b, gvl);
    
    // 向量指令依赖标量multiplier
    vfloat64m1_t v_temp = __riscv_vfmul_vf_f64m1(va, multiplier, gvl);
    vfloat64m1_t v_intermediate = __riscv_vfadd_vv_f64m1(v_temp, vb, gvl);
    
    // 步骤3: 向量归约到标量 - 计算向量元素的和
    double temp_array[ARRAY_SIZE] __attribute__((aligned(16)));
    __riscv_vse64_v_f64m1(temp_array, v_intermediate, gvl);
    
    double sum = 0.0;
    for (int i = 0; i < ARRAY_SIZE; i++) {
        sum += temp_array[i];  // 标量归约：sum依赖于向量结果
    }
    
    printf("Vector reduction result (sum): %.2f\n", sum);
    
    // 步骤4: 标量计算新的缩放因子
    double avg = sum / ARRAY_SIZE;
    double final_scale = avg * 0.1;  // 标量依赖于归约结果
    
    printf("Average: %.2f, Final scale: %.2f\n", avg, final_scale);
    
    // 步骤5: 最终向量操作 - 依赖于标量归约的结果
    vfloat64m1_t v_final = __riscv_vfmul_vf_f64m1(v_intermediate, final_scale, gvl);
    __riscv_vse64_v_f64m1(result, v_final, gvl);
    
    // 验证结果
    printf("Final results:\n");
    int all_correct = 1;
    for (int i = 0; i < ARRAY_SIZE; i++) {
        double expected = (a[i] * multiplier + b[i]) * final_scale;
        double diff = result[i] - expected;
        printf("result[%d] = %.6f, expected = %.6f", i, result[i], expected);
        if (diff < -0.0001 || diff > 0.0001) {
            printf(" [FAIL]\n");
            all_correct = 0;
        } else {
            printf(" [OK]\n");
        }
    }
    
    if (all_correct) {
        printf("✓ Test 3 PASSED\n\n");
    } else {
        printf("✗ Test 3 FAILED\n\n");
    }
}

// 测试用例4: 条件分支中的标量-向量依赖
// 场景：标量条件判断 -> 影响向量操作的执行路径
void test_conditional_scalar_vector_dependency() {
    printf("=== Test 4: Conditional scalar-vector dependency ===\n");
    
    double a[ARRAY_SIZE] __attribute__((aligned(16)));
    double result[ARRAY_SIZE] __attribute__((aligned(16)));
    
    // 初始化数组
    for (int i = 0; i < ARRAY_SIZE; i++) {
        a[i] = i + 1.0;
    }
    
    // 标量条件计算
    double threshold_base = 8.0;
    double threshold_offset = 2.0;
    double threshold = threshold_base + threshold_offset;  // 标量依赖
    
    // 标量比较决定向量操作的类型
    int use_multiplication = (threshold > 9.0) ? 1 : 0;  // 标量依赖于threshold
    
    printf("Threshold: %.2f, Use multiplication: %s\n", 
           threshold, use_multiplication ? "Yes" : "No");
    
    // 向量操作 - 依赖于标量条件判断的结果
    size_t gvl = __riscv_vsetvl_e64m1(ARRAY_SIZE);
    vfloat64m1_t va = __riscv_vle64_v_f64m1(a, gvl);
    vfloat64m1_t vresult;
    
    if (use_multiplication) {
        // 路径1: 向量乘法操作
        vresult = __riscv_vfmul_vf_f64m1(va, 2.5, gvl);
        printf("Using vector multiplication path\n");
    } else {
        // 路径2: 向量加法操作
        vresult = __riscv_vfadd_vf_f64m1(va, 5.0, gvl);
        printf("Using vector addition path\n");
    }
    
    __riscv_vse64_v_f64m1(result, vresult, gvl);
    
    // 验证结果
    printf("Results:\n");
    int all_correct = 1;
    for (int i = 0; i < ARRAY_SIZE; i++) {
        double expected = use_multiplication ? (a[i] * 2.5) : (a[i] + 5.0);
        double diff = result[i] - expected;
        printf("a[%d] = %.2f, result[%d] = %.2f, expected = %.2f", 
               i, a[i], i, result[i], expected);
        if (diff < -0.0001 || diff > 0.0001) {
            printf(" [FAIL]\n");
            all_correct = 0;
        } else {
            printf(" [OK]\n");
        }
    }
    
    if (all_correct) {
        printf("✓ Test 4 PASSED\n\n");
    } else {
        printf("✗ Test 4 FAILED\n\n");
    }
}

int main() {
    printf("=== RISC-V Vector-Scalar Dependency Test Suite ===\n");
    printf("Testing various scenarios where vector instructions depend on scalar computations\n");
    printf("Array size: %d\n\n", ARRAY_SIZE);
    
    // 运行所有测试用例
    test_vector_depends_on_scalar();
    test_vector_memory_depends_on_scalar_address();
    test_mixed_scalar_vector_dependency_chain();
    test_conditional_scalar_vector_dependency();
    
    printf("=== All tests completed ===\n");
    
    return 0;
}