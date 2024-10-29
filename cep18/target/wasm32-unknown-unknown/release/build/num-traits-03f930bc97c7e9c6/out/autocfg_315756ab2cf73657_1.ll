; ModuleID = 'autocfg_315756ab2cf73657_1.568a44b3-cgu.0'
source_filename = "autocfg_315756ab2cf73657_1.568a44b3-cgu.0"
target datalayout = "e-m:e-p:32:32-p10:8:8-p20:8:8-i64:64-n32:64-S128-ni:1:10:20"
target triple = "wasm32-unknown-unknown"

@alloc_fb24ee83aa85dfb496f2abc1d22d2cb9 = private unnamed_addr constant <{ [8 x i8] }> <{ [8 x i8] c"\00\00\00\00\00\00\F0?" }>, align 8
@alloc_a2408d0ee32d75bc015427d6310f44b6 = private unnamed_addr constant <{ [8 x i8] }> <{ [8 x i8] c"\00\00\00\00\00\00\00@" }>, align 8

; core::f64::<impl f64>::total_cmp
; Function Attrs: inlinehint nounwind
define internal i8 @"_ZN4core3f6421_$LT$impl$u20$f64$GT$9total_cmp17h0d5127a58e3e0e32E"(ptr align 8 %self, ptr align 8 %other) unnamed_addr #0 {
start:
  %_23 = alloca double, align 8
  %_21 = alloca double, align 8
  %_7 = alloca i64, align 8
  %right = alloca i64, align 8
  %_4 = alloca i64, align 8
  %left = alloca i64, align 8
  %0 = alloca i8, align 1
  %self1 = load double, ptr %self, align 8, !noundef !0
  store double %self1, ptr %_21, align 8
  %rt = load double, ptr %_21, align 8, !noundef !0
  store double %rt, ptr %_4, align 8
  %1 = load i64, ptr %_4, align 8, !noundef !0
  store i64 %1, ptr %left, align 8
  %self2 = load double, ptr %other, align 8, !noundef !0
  store double %self2, ptr %_23, align 8
  %rt3 = load double, ptr %_23, align 8, !noundef !0
  store double %rt3, ptr %_7, align 8
  %2 = load i64, ptr %_7, align 8, !noundef !0
  store i64 %2, ptr %right, align 8
  %_13 = load i64, ptr %left, align 8, !noundef !0
  %_12 = ashr i64 %_13, 63
  %_10 = lshr i64 %_12, 1
  %3 = load i64, ptr %left, align 8, !noundef !0
  %4 = xor i64 %3, %_10
  store i64 %4, ptr %left, align 8
  %_18 = load i64, ptr %right, align 8, !noundef !0
  %_17 = ashr i64 %_18, 63
  %_15 = lshr i64 %_17, 1
  %5 = load i64, ptr %right, align 8, !noundef !0
  %6 = xor i64 %5, %_15
  store i64 %6, ptr %right, align 8
  %_26 = load i64, ptr %left, align 8, !noundef !0
  %_27 = load i64, ptr %right, align 8, !noundef !0
  %_25 = icmp slt i64 %_26, %_27
  br i1 %_25, label %bb1, label %bb2

bb2:                                              ; preds = %start
  %_29 = load i64, ptr %left, align 8, !noundef !0
  %_30 = load i64, ptr %right, align 8, !noundef !0
  %_28 = icmp eq i64 %_29, %_30
  br i1 %_28, label %bb3, label %bb4

bb1:                                              ; preds = %start
  store i8 -1, ptr %0, align 1
  br label %bb6

bb6:                                              ; preds = %bb5, %bb1
  %7 = load i8, ptr %0, align 1, !range !1, !noundef !0
  ret i8 %7

bb4:                                              ; preds = %bb2
  store i8 1, ptr %0, align 1
  br label %bb5

bb3:                                              ; preds = %bb2
  store i8 0, ptr %0, align 1
  br label %bb5

bb5:                                              ; preds = %bb4, %bb3
  br label %bb6
}

; autocfg_315756ab2cf73657_1::probe
; Function Attrs: nounwind
define hidden void @_ZN26autocfg_315756ab2cf73657_15probe17h730f0ae2f8f76a86E() unnamed_addr #1 {
start:
; call core::f64::<impl f64>::total_cmp
  %_1 = call i8 @"_ZN4core3f6421_$LT$impl$u20$f64$GT$9total_cmp17h0d5127a58e3e0e32E"(ptr align 8 @alloc_fb24ee83aa85dfb496f2abc1d22d2cb9, ptr align 8 @alloc_a2408d0ee32d75bc015427d6310f44b6) #2, !range !1
  ret void
}

attributes #0 = { inlinehint nounwind "target-cpu"="generic" }
attributes #1 = { nounwind "target-cpu"="generic" }
attributes #2 = { nounwind }

!0 = !{}
!1 = !{i8 -1, i8 2}
