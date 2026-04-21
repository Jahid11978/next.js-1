(globalThis["TURBOPACK"] || (globalThis["TURBOPACK"] = [])).push(["output/1do3_crates_turbopack-tests_tests_snapshot_comptime_cross-module-cycle_input_0juf-pm._.js",
"[project]/turbopack/crates/turbopack-tests/tests/snapshot/comptime/cross-module-cycle/input/other.js [test] (ecmascript)", (function(__turbopack_context__){
"use strict";

__turbopack_context__.s([
    "FOO",
    ()=>foo1
]);
var __TURBOPACK__imported__module__$5b$project$5d2f$turbopack$2f$crates$2f$turbopack$2d$tests$2f$tests$2f$snapshot$2f$comptime$2f$cross$2d$module$2d$cycle$2f$input$2f$other$2e$js__$5b$test$5d$__$28$ecmascript$29$__ = __turbopack_context__.i("[project]/turbopack/crates/turbopack-tests/tests/snapshot/comptime/cross-module-cycle/input/other.js [test] (ecmascript)");
;
function foo1(left, right) {
    // might be an infinite loop at runtime, but shouldn't hang the build
    return (0, __TURBOPACK__imported__module__$5b$project$5d2f$turbopack$2f$crates$2f$turbopack$2d$tests$2f$tests$2f$snapshot$2f$comptime$2f$cross$2d$module$2d$cycle$2f$input$2f$other$2e$js__$5b$test$5d$__$28$ecmascript$29$__["FOO"])(left, right);
}
;
}),
"[project]/turbopack/crates/turbopack-tests/tests/snapshot/comptime/cross-module-cycle/input/index.js [test] (ecmascript)", (function(__turbopack_context__){
"use strict";

__turbopack_context__.s([]);
var __TURBOPACK__imported__module__$5b$project$5d2f$turbopack$2f$crates$2f$turbopack$2d$tests$2f$tests$2f$snapshot$2f$comptime$2f$cross$2d$module$2d$cycle$2f$input$2f$other$2e$js__$5b$test$5d$__$28$ecmascript$29$__ = __turbopack_context__.i("[project]/turbopack/crates/turbopack-tests/tests/snapshot/comptime/cross-module-cycle/input/other.js [test] (ecmascript)");
;
console.log(__TURBOPACK__imported__module__$5b$project$5d2f$turbopack$2f$crates$2f$turbopack$2d$tests$2f$tests$2f$snapshot$2f$comptime$2f$cross$2d$module$2d$cycle$2f$input$2f$other$2e$js__$5b$test$5d$__$28$ecmascript$29$__["FOO"]);
}),
]);

//# sourceMappingURL=1do3_crates_turbopack-tests_tests_snapshot_comptime_cross-module-cycle_input_0juf-pm._.js.map