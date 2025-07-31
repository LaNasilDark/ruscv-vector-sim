/*************************************************************************
* Vectorized matrixmul Kernel
*************************************************************************/

#include <stdlib.h>
#include <stdio.h>
#include <math.h>
#include <assert.h>
#include <stdbool.h>

#define DATA_TYPE 
typedef double data_t;
//#define USE_RISCV_VECTOR
#ifdef USE_RISCV_VECTOR
#include <riscv_vector.h>
#include "../../common/vector_defines.h"

#define min(a, b) ((a) < (b) ? (a) : (b))

void matrixmul_intrinsics(data_t *a, data_t *b, data_t *c, int n, int m, int p) {
    //m5_reset_stats(0, 0);    // 重置性能计数器

    for (size_t i = 0; i < m; i++) {
        for (size_t j = 0; j < n; j++) {
            size_t gvl = _MMR_VSETVL_E64M1(p);
            vfloat64m1_t vprod = _MM_SET_f64(0, gvl); 
            vfloat64m1_t vsum  = _MM_SET_f64(0, gvl);

            for (size_t k = 0; k < p; k += gvl){
                gvl = _MMR_VSETVL_E64M1(p - k);
                 
                // Matrix A row
                vfloat64m1_t va  = _MM_LOAD_f64(&a[i*p+k], gvl); 
                // Matrix B column
                vfloat64m1_t vb = _MM_LOAD_f64(&b[j*p+k], gvl);
                
                // A[0]*B[0], A[1]*B[1],... A[n]*B[n]
                vprod  = _MM_MACC_f64(vprod,va, vb, gvl); 
  
            }//k
            gvl = _MMR_VSETVL_E64M1(p);
            vsum   = _MM_REDSUM_f64(vprod,vsum, gvl);
            c[i*n+j] = _MM_VGETFIRST_f64(vsum);
        }//j
    }//i
    //m5_dump_stats(0, 0);     // 保存当前统计信息
}

void matrixmul_intrinsics_tiled(data_t *a, data_t *b, data_t *c, int n, int m, int p) {
    //m5_reset_stats(0, 0);    // 重置性能计数器

    const size_t BLOCK_SIZE_M = 32;
    const size_t BLOCK_SIZE_N = 32;
    const size_t BLOCK_SIZE_P = 32;
    const size_t GLOBAL_VL = _MMR_VSETVL_E64M1(p);
    for(size_t i=0;i<m;i+= BLOCK_SIZE_M) {
        size_t i_end = m < i + BLOCK_SIZE_M ? m : i + BLOCK_SIZE_M;
        for(size_t j=0;j < n;j += BLOCK_SIZE_N){
            size_t j_end = n < j + BLOCK_SIZE_N ? n : j + BLOCK_SIZE_N;
            for(size_t k = 0 ; k < p ; k += BLOCK_SIZE_P) {
                for(size_t ii = i ; ii < i_end; ++ii) {
                    for(size_t jj = j ; jj < j_end ; ++jj) {
                        size_t k_end = p < k + BLOCK_SIZE_P ? p : k + BLOCK_SIZE_P;


                        vfloat64m1_t vprod = _MM_SET_f64(0, GLOBAL_VL); 
                        
                        // 保证k_end和kk的距离是要GLOBAL VL的整数倍
                        for(size_t kk = k ; kk < k_end ; kk += GLOBAL_VL) {

                            // Matrix A row
                            vfloat64m1_t va  = _MM_LOAD_f64(&a[ii*p+kk], GLOBAL_VL); 
                            // Matrix B(before the transposition) column
                            vfloat64m1_t vb = _MM_LOAD_f64(&b[jj*p+kk], GLOBAL_VL);
                            
                            // A[0]*B[0], A[1]*B[1],... A[n]*B[n]
                            vprod  = _MM_MACC_f64(vprod,va, vb, GLOBAL_VL); 

                        }
                        vfloat64m1_t vsum  = _MM_SET_f64(0, GLOBAL_VL);
                        vsum   = _MM_REDSUM_f64(vprod,vsum, GLOBAL_VL);
                        c[ii*n+jj] += _MM_VGETFIRST_f64(vsum);

                    }
                }
            }
        }
    }

    //m5_dump_stats(0, 0);     // 保存当前统计信息
}
#else // !USE_RISCV_VECTOR

void matmul_serial(data_t *a, data_t *b, data_t *c, int n, int m, int p) {
    for (int i = 0; i < m; ++i)
        for (int j = 0; j < n; ++j) {
            c[i * n + j] = 0;
            for (int k = 0; k < p; ++k) {
                c[i * n + j] += a[i * p + k] * b[k * n + j];
            }
        }
}

#endif


bool compare( size_t dm, size_t dn, data_t *a ,data_t *b) {
    bool result = false;
    for (int i = 0; i < dm; i++) {
        for (int j = 0; j < dn; j++) {
            if(a[i*dn+j] != b[i*dn+j]) {
              result = true;
            }
        }
 
    }
    return result;
}
