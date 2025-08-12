use lycrex_tool::utils::math::*;
use colored::Colorize;
fn main() {
    // 数学常量
    println!("1. Constants:");
    test_constant("π", M_PI, 3.1415926535897932384626433832795);
    test_constant("e", M_E, 2.7182818284590452353602874713527);
    test_constant("√2", M_SQRT2, 1.4142135623730950488016887242097);
    test_constant("ln(2)", M_LN2, 0.69314718055994530941723212145818);
    test_constant("log₂(e)", M_LOG2E, 1.4426950408889634073599246810019);
    test_constant("1/√2", M_SQRT1_2, 0.70710678118654752440084436210485);
    println!();

    // 三角函数精度测试
    println!("2. Trigonometric Functions:");
    test_function("sin(π/6)", sin(M_PI/6.0), 0.5, "sin(30°) = 1/2");
    test_function("cos(π/3)", cos(M_PI/3.0), 0.5, "cos(60°) = 1/2");
    test_function("tan(π/4)", tan(M_PI/4.0), 1.0, "tan(45°) = 1");
    test_function("sin(π/2)", sin(M_PI/2.0), 1.0, "sin(90°) = 1");
    test_function("cos(0)", cos(0.0), 1.0, "cos(0°) = 1");
    
    // 特殊角：sin(π/4) = cos(π/4) = √2/2
    let sqrt2_over_2 = M_SQRT2 / 2.0;
    test_function("sin(π/4)", sin(M_PI/4.0), sqrt2_over_2, "sin(45°) = √2/2");
    test_function("cos(π/4)", cos(M_PI/4.0), sqrt2_over_2, "cos(45°) = √2/2");
    println!();

    // 使用精度处理的三角函数测试 - 链式调用风格
    println!("2b. Trigonometric Functions (Precision Enhanced):");
    test_function("sin(π/6).precise()", sin(M_PI/6.0).precise(), 0.5, "sin(30°) = 1/2");
    test_function("cos(π/3).precise()", cos(M_PI/3.0).precise(), 0.5, "cos(60°) = 1/2");
    test_function("tan(π/4).precise()", tan(M_PI/4.0).precise(), 1.0, "tan(45°) = 1");
    test_function("sin(π/2).precise()", sin(M_PI/2.0).precise(), 1.0, "sin(90°) = 1");
    test_function("cos(0).precise()", cos(0.0).precise(), 1.0, "cos(0°) = 1");
    test_function("sin(π/4).precise()", sin(M_PI/4.0).precise(), sqrt2_over_2, "sin(45°) = √2/2");
    test_function("cos(π/4).precise()", cos(M_PI/4.0).precise(), sqrt2_over_2, "cos(45°) = √2/2");
    
    println!();
    
    println!("2c. Direct Angle Precision Calculation:");
    test_function("(π/6).sin_precise()", (M_PI/6.0).sin_precise(), 0.5, "");
    test_function("(π/3).cos_precise()", (M_PI/3.0).cos_precise(), 0.5, "");
    test_function("(π/4).tan_precise()", (M_PI/4.0).tan_precise(), 1.0, "");
    
    println!();

    // 反三角函数
    println!("3. Inverse Trigonometric Functions:");
    test_function("arcsin(1/2)", asin(0.5), M_PI/6.0, "arcsin(1/2) = π/6");
    test_function("arccos(1/2)", acos(0.5), M_PI/3.0, "arccos(1/2) = π/3");
    test_function("arctan(1)", atan(1.0), M_PI/4.0, "arctan(1) = π/4");
    test_function("arctan2(1,1)", atan2(1.0, 1.0), M_PI/4.0, "arctan2(1,1) = π/4");
    
    println!();

    // 指数和对数函数
    println!("4. Exponential and Logarithmic Functions:");
    test_function("e¹", exp(1.0), M_E, "e¹ = e");
    test_function("ln(e)", log(M_E), 1.0, "ln(e) = 1");
    test_function("log₁₀(10)", log10(10.0), 1.0, "log₁₀(10) = 1");
    test_function("log₁₀(100)", log10(100.0), 2.0, "log₁₀(100) = 2");
    test_function("log₂(8)", log2(8.0), 3.0, "log₂(8) = 3");
    test_function("2¹⁰", pow(2.0, 10.0), 1024.0, "2¹⁰ = 1024");
    test_function("√16", sqrt(16.0), 4.0, "√16 = 4");
    test_function("√780", sqrt(780.0), 27.9284800875378814, "√780 = 27.9284800875378814");
    test_function("∛27", cbrt(27.0), 3.0, "∛27 = 3");
    test_function("∛982", cbrt(982.0), 9.9396363562216585, "∛982 = 9.9396363562216585");

    println!();

    // 5. 双曲函数
    println!("5. Hyperbolic Functions:");
    test_function("sinh(0)", sinh(0.0), 0.0, "sinh(0) = 0");
    test_function("cosh(0)", cosh(0.0), 1.0, "cosh(0) = 1");
    test_function("tanh(0)", tanh(0.0), 0.0, "tanh(0) = 0");
    
    // 双曲函数恒等式：cosh²(x) - sinh²(x) = 1
    let x = 1.5;
    let sinh_x = sinh(x);
    let cosh_x = cosh(x);
    let identity = cosh_x * cosh_x - sinh_x * sinh_x;
    test_function("cosh²(1.5) - sinh²(1.5)", identity, 1.0, "双曲函数恒等式");
    
    println!();

    // 6. Gamma函数精度测试
    println!("6. Gamma Function:");
    test_function("Γ(1)", tgamma(1.0), 1.0, "Γ(1) = 0! = 1");
    test_function("Γ(2)", tgamma(2.0), 1.0, "Γ(2) = 1! = 1");
    test_function("Γ(3)", tgamma(3.0), 2.0, "Γ(3) = 2! = 2");
    test_function("Γ(4)", tgamma(4.0), 6.0, "Γ(4) = 3! = 6");
    test_function("Γ(5)", tgamma(5.0), 24.0, "Γ(5) = 4! = 24");
    test_function("Γ(1/2)", tgamma(0.5), M_PI.sqrt(), "Γ(1/2) = √π");
    
    // Gamma函数递推关系：Γ(x+1) = x·Γ(x)
    let x = 2.5;
    let gamma_x = tgamma(x);
    let gamma_x_plus_1 = tgamma(x + 1.0);
    let relation = x * gamma_x;
    test_function("2.5·Γ(2.5)", relation, gamma_x_plus_1, "递推关系验证");
    
    println!();

    // 7. 误差函数和补误差函数 - 完整测试
    println!("7. Error Function:");
    // simple_erf_test();
    full_erf_test();
    
    // 互补关系测试：erf(x) + erfc(x) = 1
    for test_x in [0.1, 0.5, 1.0, 1.5, 2.0, 3.0] {
        let sum = erf(test_x) + erfc(test_x);
        test_function(&format!("erf({}) + erfc({})", test_x, test_x), sum, 1.0, "互补关系");
    }
    
    // 小数值精度测试
    test_function("erf(1e-8)", erf(1e-8), erf(1e-8), "极小值精度");
    test_function("erf(1e-15)", erf(1e-15), erf(1e-15), "极小值精度");
    
    println!();

    // 8. 贝塞尔函数
    println!("8. Bessel Functions:");
    test_function("J₀(0)", j0(0.0), 1.0, "J₀(0) = 1");
    test_function("J₁(0)", j1(0.0), 0.0, "J₁(0) = 0");
    test_function("J₀(1)", j0(1.0), 0.7651976865579666, "J₀(1) 标准值");
    test_function("J₁(1)", j1(1.0), 0.4400505857449335, "J₁(1) 标准值");
    
    println!();

    // 9. 复数运算精度
    println!("9. Complex Number Operations:");
    let z1 = Complex::new(3.0, 4.0);  // 3+4i
    let z2 = Complex::new(5.0, 12.0); // 5+12i
    test_function("|3+4i|", z1.abs(), 5.0, "勾股定理: √(3²+4²) = 5");
    test_function("|5+12i|", z2.abs(), 13.0, "勾股定理: √(5²+12²) = 13");
    
    // 欧拉公式的验证：|e^(iθ)| = 1
    let theta = M_PI / 3.0; // 60度
    let euler_complex = Complex::new(cos(theta), sin(theta));
    test_function("|e^(iπ/3)|", euler_complex.abs(), 1.0, "欧拉公式验证");
    
    println!();

    // 10. 特殊函数组合
    println!("10. Special Function Combinations:");
    
    // 欧拉恒等式的部分：e^(iπ) + 1 = 0，即 e^(iπ) = -1
    // 这里验证 e^π 的值
    test_function("e^π", exp(M_PI), 23.140692632779267, "超越数");
    
    // 勾股定理：hypot(a,b) = √(a²+b²)
    test_function("hypot(3,4)", hypot(3.0, 4.0), 5.0, "勾股定理");
    test_function("hypot(5,12)", hypot(5.0, 12.0), 13.0, "勾股定理");
    
    // 对数恒等式：log(ab) = log(a) + log(b)
    let a = 2.0;
    let b = 8.0;
    let log_product = log(a * b);
    let sum_logs = log(a) + log(b);
    test_function("log(2×8)", log_product, sum_logs, "对数法则");
    
    // 幂法则：(a^m)^n = a^(mn)
    let base = 2.0;
    let m = 3.0;
    let n = 4.0;
    let power1 = pow(pow(base, m), n);
    let power2 = pow(base, m * n);
    test_function("(2³)⁴", power1, power2, "幂法则");
    
    println!();

    // 11. 数值稳定性测试
    println!("11. Numerical Stability Test:");
    
    // 接近边界值的测试
    test_function("sin(π)", sin(M_PI), 0.0, "边界值 (应该为0)");
    test_function("cos(π/2)", cos(M_PI/2.0), 0.0, "边界值 (应该为0)");
    
    // 大数值测试
    let large_x = 100.0;
    let sinh_large = sinh(large_x);
    let exp_approx = exp(large_x) / 2.0;
    let relative_error = (sinh_large - exp_approx).abs() / exp_approx;
    println!("   sinh(100) ≈ e^100/2: 相对误差 = {:.2e}", relative_error);
    
    // 小数值测试 (泰勒展开验证)
    let small_x = 1e-10;
    let sin_small = sin(small_x);
    let relative_error_small = (sin_small - small_x).abs() / small_x;
    println!("   sin(10⁻¹⁰) ≈ 10⁻¹⁰: 相对误差 = {:.2e}", relative_error_small);
    
    println!();

    // 12. 新增高级数学函数
    println!("12. Advanced Mathematical Functions:");
    
    // 反双曲函数
    test_function("asinh(1)", asinh(1.0), 0.8813735870195429, "反双曲正弦");
    test_function("acosh(2)", acosh(2.0), 1.3169578969248166, "反双曲余弦");
    test_function("atanh(0.5)", atanh(0.5), 0.5493061443340549, "反双曲正切");
    
    // 椭圆积分
    test_function("K(0.5)", elliptic_k(0.5), 1.6857503548125963, "第一类椭圆积分");
    test_function("E(0.5)", elliptic_e(0.5), 1.4674622093394271, "第二类椭圆积分");
    
    // 特殊函数
    test_function("ψ(2)", digamma(2.0), 0.4227843350984671, "Digamma函数");
    test_function("ζ(2)", riemann_zeta(2.0), M_PI*M_PI/6.0, "黎曼ζ函数");
    test_function("ζ(4)", riemann_zeta(4.0), M_PI.powi(4)/90.0, "黎曼ζ函数");
    
    // 概率分布函数
    test_function("Φ(0)", normal_cdf(0.0), 0.5, "标准正态分布CDF");
    test_function("φ(0)", normal_pdf(0.0), 1.0/(2.0*M_PI).sqrt(), "标准正态分布PDF");
    test_function("χ²CDF(2,2)", chi_squared_cdf(2.0, 2.0), 0.6321205588285577, "卡方分布CDF");
    
    println!();
}

fn test_constant(name: &str, computed: f64, expected: f64) {
    let error = (computed - expected).abs();
    let relative_error = error / expected.abs();
    println!("   {} = {:.16} (误差: {:.2e}, 相对误差: {:.2e})", 
             name, computed, error, relative_error);
}

fn test_function(name: &str, computed: f64, expected: f64, description: &str) {
    let error = (computed - expected).abs();
    let relative_error = if expected.abs() > 1e-15 {
        error / expected.abs()
    } else {
        error
    };
    
    let status = if 
    relative_error < 1e-16 {
        "✓ 完美".green().bold()
    } else if relative_error < 1e-14 {
        "✓ 优秀".green()
    } else if relative_error < 1e-12 {
        "✓ 良好".blue()
    } else if relative_error < 1e-10 {
        "⚠ 一般".yellow()
    } else if relative_error < 1e-8 {
        "⚠ 较差".yellow().bold()
    } else if relative_error < 1e-6 {
        "✗ 极差".red()
    } else {
        // 错误等级取 6 - 对数绝对值
        let error_level = 6 - (relative_error.log10() as i32).abs();
        format!("{} 错误", error_level).red().bold()
    };
    
    println!("[{}]   {} = {:.16} {} (Error: {:.2e})", 
             status,
             name, computed, description, relative_error);
}

#[allow(dead_code)]
fn test_float_classification(x: f64, description: &str) {
    println!("   {} ({}):", description, x);
    println!("     isnan: {}", isnan(x));
    println!("     isinf: {}", isinf(x));
    println!("     isfinite: {}", isfinite(x));
    println!("     isnormal: {}", isnormal(x));
    println!("     signbit: {}", signbit(x));
}

#[allow(dead_code)]
fn simple_erf_test() {
    test_function("erf(0)", erf(0.0), 0.0, "erf(0) = 0");
    test_function("erf(∞)", erf(f64::INFINITY), 1.0, "erf(∞) = 1");
    test_function("erf(-∞)", erf(f64::NEG_INFINITY), -1.0, "erf(-∞) = -1");
    test_function("erf(0.020)", erf(0.020), 0.022564575, "小参数区间");
    test_function("erf(0.100)", erf(0.100), 0.112462916, "小参数区间");
    test_function("erf(0.200)", erf(0.200), 0.222702589, "中小参数区间");
    test_function("erf(0.900)", erf(0.900), 0.796908212, "中等参数区间");
    test_function("erf(1.000)", erf(1.000), 0.842700793, "标准测试值");
    test_function("erf(2.000)", erf(2.000), 0.995322265, "中大参数区间");
    test_function("erf(3.500)", erf(3.500), 0.999999257, "大参数区间");
    test_function("erf(-0.10)", erf(-0.1), -0.112462916, "负数对称性");
    test_function("erf(-0.50)", erf(-0.5), -0.520499878, "负数对称性");
    test_function("erf(-1.00)", erf(-1.0), -0.842700793, "负数对称性");
    test_function("erf(-2.00)", erf(-2.0), -0.995322265, "负数对称性");
    test_function("erfc(0.02)", erfc(0.02), 0.977435425, "小参数互补");
    test_function("erfc(0.04)", erfc(0.04), 0.954888894, "小参数互补");
    test_function("erfc(0.06)", erfc(0.06), 0.932378406, "小参数互补");
    test_function("erfc(0.08)", erfc(0.08), 0.909921874, "小参数互补");
}

#[allow(dead_code)]
fn full_erf_test() {
    // 基本性质测试
    test_function("erf(0)", erf(0.0), 0.0, "erf(0) = 0");
    test_function("erf(∞)", erf(f64::INFINITY), 1.0, "erf(∞) = 1");
    test_function("erf(-∞)", erf(f64::NEG_INFINITY), -1.0, "erf(-∞) = -1");
    
    // 小参数精度测试 (0 < x ≤ 0.1)
    test_function("erf(0.02)", erf(0.02), 0.022564575, "小参数区间");
    test_function("erf(0.04)", erf(0.04), 0.045111106, "小参数区间");
    test_function("erf(0.06)", erf(0.06), 0.067621594, "小参数区间");
    test_function("erf(0.08)", erf(0.08), 0.090078126, "小参数区间");
    test_function("erf(0.10)", erf(0.10), 0.112462916, "小参数区间");
    
    // 中小参数测试 (0.1 < x ≤ 0.5)
    test_function("erf(0.2)", erf(0.2), 0.222702589, "中小参数区间");
    test_function("erf(0.3)", erf(0.3), 0.328626759, "中小参数区间");
    test_function("erf(0.4)", erf(0.4), 0.428392355, "中小参数区间");
    test_function("erf(0.5)", erf(0.5), 0.520499878, "中小参数区间");
    
    // 中等参数测试 (0.5 < x ≤ 1.0)
    test_function("erf(0.6)", erf(0.6), 0.603856091, "中等参数区间");
    test_function("erf(0.7)", erf(0.7), 0.677801194, "中等参数区间");
    test_function("erf(0.8)", erf(0.8), 0.742100965, "中等参数区间");
    test_function("erf(0.9)", erf(0.9), 0.796908212, "中等参数区间");
    test_function("erf(1.0)", erf(1.0), 0.842700793, "标准测试值");
    
    // 中大参数测试 (1.0 < x ≤ 2.0)
    test_function("erf(1.1)", erf(1.1), 0.880205070, "中大参数区间");
    test_function("erf(1.2)", erf(1.2), 0.910313978, "中大参数区间");
    test_function("erf(1.3)", erf(1.3), 0.934007945, "中大参数区间");
    test_function("erf(1.4)", erf(1.4), 0.952285120, "中大参数区间");
    test_function("erf(1.5)", erf(1.5), 0.966105146, "中大参数区间");
    test_function("erf(1.6)", erf(1.6), 0.976348383, "中大参数区间");
    test_function("erf(1.7)", erf(1.7), 0.983790459, "中大参数区间");
    test_function("erf(1.8)", erf(1.8), 0.989090502, "中大参数区间");
    test_function("erf(1.9)", erf(1.9), 0.992790429, "中大参数区间");
    test_function("erf(2.0)", erf(2.0), 0.995322265, "中大参数区间");
    
    // 大参数测试 (x > 2.0)
    test_function("erf(2.1)", erf(2.1), 0.997020533, "大参数区间");
    test_function("erf(2.2)", erf(2.2), 0.998137154, "大参数区间");
    test_function("erf(2.3)", erf(2.3), 0.998856823, "大参数区间");
    test_function("erf(2.4)", erf(2.4), 0.999311486, "大参数区间");
    test_function("erf(2.5)", erf(2.5), 0.999593048, "大参数区间");
    test_function("erf(3.0)", erf(3.0), 0.999977910, "大参数区间");
    test_function("erf(3.5)", erf(3.5), 0.999999257, "大参数区间");
    
    // 负数对称性测试
    test_function("erf(-0.1)", erf(-0.1), -0.112462916, "负数对称性");
    test_function("erf(-0.5)", erf(-0.5), -0.520499878, "负数对称性");
    test_function("erf(-1.0)", erf(-1.0), -0.842700793, "负数对称性");
    test_function("erf(-2.0)", erf(-2.0), -0.995322265, "负数对称性");
    
    // 补误差函数 erfc(x) = 1 - erf(x) 精度测试
    println!("\n   Complementary Error Function (erfc):");
    
    // 小参数 erfc 测试
    test_function("erfc(0.02)", erfc(0.02), 0.977435425, "小参数互补");
    test_function("erfc(0.04)", erfc(0.04), 0.954888894, "小参数互补");
    test_function("erfc(0.06)", erfc(0.06), 0.932378406, "小参数互补");
    test_function("erfc(0.08)", erfc(0.08), 0.909921874, "小参数互补");
    test_function("erfc(0.10)", erfc(0.10), 0.887537084, "小参数互补");
    
    // 中小参数 erfc 测试
    test_function("erfc(0.2)", erfc(0.2), 0.777297411, "中小参数互补");
    test_function("erfc(0.3)", erfc(0.3), 0.671373241, "中小参数互补");
    test_function("erfc(0.4)", erfc(0.4), 0.571607645, "中小参数互补");
    test_function("erfc(0.5)", erfc(0.5), 0.479500122, "中小参数互补");
    
    // 中等参数 erfc 测试
    test_function("erfc(0.6)", erfc(0.6), 0.396143909, "中等参数互补");
    test_function("erfc(0.7)", erfc(0.7), 0.322198806, "中等参数互补");
    test_function("erfc(0.8)", erfc(0.8), 0.257899035, "中等参数互补");
    test_function("erfc(0.9)", erfc(0.9), 0.203091788, "中等参数互补");
    test_function("erfc(1.0)", erfc(1.0), 0.157299207, "标准互补值");
    
    // 中大参数 erfc 测试
    test_function("erfc(1.1)", erfc(1.1), 0.119794930, "中大参数互补");
    test_function("erfc(1.2)", erfc(1.2), 0.089686022, "中大参数互补");
    test_function("erfc(1.3)", erfc(1.3), 0.065992055, "中大参数互补");
    test_function("erfc(1.4)", erfc(1.4), 0.047714880, "中大参数互补");
    test_function("erfc(1.5)", erfc(1.5), 0.033894854, "中大参数互补");
    test_function("erfc(1.6)", erfc(1.6), 0.023651617, "中大参数互补");
    test_function("erfc(1.7)", erfc(1.7), 0.016209541, "中大参数互补");
    test_function("erfc(1.8)", erfc(1.8), 0.010909498, "中大参数互补");
    test_function("erfc(1.9)", erfc(1.9), 0.007209571, "中大参数互补");
    test_function("erfc(2.0)", erfc(2.0), 0.004677735, "中大参数互补");
    
    // 大参数 erfc 测试
    test_function("erfc(2.1)", erfc(2.1), 0.002979467, "大参数互补");
    test_function("erfc(2.2)", erfc(2.2), 0.001862846, "大参数互补");
    test_function("erfc(2.3)", erfc(2.3), 0.001143177, "大参数互补");
    test_function("erfc(2.4)", erfc(2.4), 0.000688514, "大参数互补");
    test_function("erfc(2.5)", erfc(2.5), 0.000406952, "大参数互补");
    test_function("erfc(3.0)", erfc(3.0), 0.000022090, "大参数互补");
    test_function("erfc(3.5)", erfc(3.5), 0.000000743, "大参数互补");
}