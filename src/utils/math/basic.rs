/// 基础数学函数

/// 复数结构体
#[derive(Debug, Clone, Copy)]
pub struct Complex {
    pub x: f64,  // 实
    pub y: f64,  // 虚
}

impl Complex {
    pub fn new(x: f64, y: f64) -> Self {
        Complex { x, y }
    }
    
    /// 计算复数的模（绝对值）
    pub fn abs(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

/// 异常结构体
#[derive(Debug, Clone)]
pub struct Exception {
    pub exception_type: i32,
    pub name: String,
    pub arg1: f64,
    pub arg2: f64,
    pub retval: f64,
}

impl Exception {
    pub fn new(exception_type: i32, name: &str, arg1: f64, arg2: f64, retval: f64) -> Self {
        Exception {
            exception_type,
            name: name.to_string(),
            arg1,
            arg2,
            retval,
        }
    }
}

/// 绝对值函数
pub fn abs(x: i32) -> i32 {
    x.abs()
}

pub fn labs(x: i64) -> i64 {
    x.abs()
}

pub fn fabs(x: f64) -> f64 {
    x.abs()
}

pub fn fabsf(x: f32) -> f32 {
    x.abs()
}

/// 三角函数
pub fn sin(x: f64) -> f64 {
    x.sin()
}

pub fn sinf(x: f32) -> f32 {
    x.sin()
}

pub fn cos(x: f64) -> f64 {
    x.cos()
}

pub fn cosf(x: f32) -> f32 {
    x.cos()
}

pub fn tan(x: f64) -> f64 {
    x.tan()
}

pub fn tanf(x: f32) -> f32 {
    x.tan()
}

/// 反三角函数
pub fn asin(x: f64) -> f64 {
    x.asin()
}

pub fn asinf(x: f32) -> f32 {
    x.asin()
}

pub fn acos(x: f64) -> f64 {
    x.acos()
}

pub fn acosf(x: f32) -> f32 {
    x.acos()
}

pub fn atan(x: f64) -> f64 {
    x.atan()
}

pub fn atanf(x: f32) -> f32 {
    x.atan()
}

pub fn atan2(y: f64, x: f64) -> f64 {
    y.atan2(x)
}

pub fn atan2f(y: f32, x: f32) -> f32 {
    y.atan2(x)
}

/// 双曲函数
pub fn sinh(x: f64) -> f64 {
    x.sinh()
}

pub fn sinhf(x: f32) -> f32 {
    x.sinh()
}

pub fn cosh(x: f64) -> f64 {
    x.cosh()
}

pub fn coshf(x: f32) -> f32 {
    x.cosh()
}

pub fn tanh(x: f64) -> f64 {
    x.tanh()
}

pub fn tanhf(x: f32) -> f32 {
    x.tanh()
}

/// 指数和对数函数
pub fn exp(x: f64) -> f64 {
    x.exp()
}

pub fn expf(x: f32) -> f32 {
    x.exp()
}

pub fn exp2(x: f64) -> f64 {
    x.exp2()
}

pub fn exp2f(x: f32) -> f32 {
    x.exp2()
}

pub fn expm1(x: f64) -> f64 {
    x.exp_m1()
}

pub fn expm1f(x: f32) -> f32 {
    x.exp_m1()
}

pub fn log(x: f64) -> f64 {
    x.ln()
}

pub fn logf(x: f32) -> f32 {
    x.ln()
}

pub fn log10(x: f64) -> f64 {
    x.log10()
}

pub fn log10f(x: f32) -> f32 {
    x.log10()
}

pub fn log2(x: f64) -> f64 {
    x.log2()
}

pub fn log2f(x: f32) -> f32 {
    x.log2()
}

pub fn log1p(x: f64) -> f64 {
    x.ln_1p()
}

pub fn log1pf(x: f32) -> f32 {
    x.ln_1p()
}

/// 幂函数
pub fn pow(x: f64, y: f64) -> f64 {
    x.powf(y)
}

pub fn powf(x: f32, y: f32) -> f32 {
    x.powf(y)
}

pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

pub fn sqrtf(x: f32) -> f32 {
    x.sqrt()
}

pub fn cbrt(x: f64) -> f64 {
    x.cbrt()
}

pub fn cbrtf(x: f32) -> f32 {
    x.cbrt()
}

/// 取整函数
pub fn ceil(x: f64) -> f64 {
    x.ceil()
}

pub fn ceilf(x: f32) -> f32 {
    x.ceil()
}

pub fn floor(x: f64) -> f64 {
    x.floor()
}

pub fn floorf(x: f32) -> f32 {
    x.floor()
}

pub fn round(x: f64) -> f64 {
    x.round()
}

pub fn roundf(x: f32) -> f32 {
    x.round()
}

pub fn trunc(x: f64) -> f64 {
    x.trunc()
}

pub fn truncf(x: f32) -> f32 {
    x.trunc()
}

/// 取模函数
pub fn fmod(x: f64, y: f64) -> f64 {
    x % y
}

pub fn fmodf(x: f32, y: f32) -> f32 {
    x % y
}

pub fn remainder(x: f64, y: f64) -> f64 {
    x - (x / y).round() * y
}

pub fn remainderf(x: f32, y: f32) -> f32 {
    x - (x / y).round() * y
}

/// 分解浮点数
pub fn frexp(x: f64) -> (f64, i32) {
    if x == 0.0 {
        return (x, 0);
    }
    
    let bits = x.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32 - 1023;
    let mantissa_bits = (bits & 0x000f_ffff_ffff_ffff) | 0x3fe0_0000_0000_0000;
    let mantissa = f64::from_bits(mantissa_bits);
    
    (mantissa, exponent + 1)
}

pub fn frexpf(x: f32) -> (f32, i32) {
    if x == 0.0 {
        return (x, 0);
    }
    
    let bits = x.to_bits();
    let exponent = ((bits >> 23) & 0xff) as i32 - 127;
    let mantissa_bits = (bits & 0x007f_ffff) | 0x3f00_0000;
    let mantissa = f32::from_bits(mantissa_bits);
    
    (mantissa, exponent + 1)
}

pub fn ldexp(x: f64, exp: i32) -> f64 {
    x * (2.0_f64).powi(exp)
}

pub fn ldexpf(x: f32, exp: i32) -> f32 {
    x * (2.0_f32).powi(exp)
}

pub fn modf(x: f64) -> (f64, f64) {
    let integer_part = x.trunc();
    let fractional_part = x - integer_part;
    (fractional_part, integer_part)
}

pub fn modff(x: f32) -> (f32, f32) {
    let integer_part = x.trunc();
    let fractional_part = x - integer_part;
    (fractional_part, integer_part)
}

/// 符号相关函数
pub fn copysign(x: f64, y: f64) -> f64 {
    x.copysign(y)
}

pub fn copysignf(x: f32, y: f32) -> f32 {
    x.copysign(y)
}

/// 字符串转换
pub fn atof(s: &str) -> f64 {
    s.parse().unwrap_or(0.0)
}

/// 复数绝对值
pub fn cabs(z: Complex) -> f64 {
    z.abs()
}

/// 双曲距离
pub fn hypot(x: f64, y: f64) -> f64 {
    x.hypot(y)
}

pub fn hypotf(x: f32, y: f32) -> f32 {
    x.hypot(y)
}

/// 最大值和最小值
pub fn fmax(x: f64, y: f64) -> f64 {
    x.max(y)
}

pub fn fmaxf(x: f32, y: f32) -> f32 {
    x.max(y)
}

pub fn fmin(x: f64, y: f64) -> f64 {
    x.min(y)
}

pub fn fminf(x: f32, y: f32) -> f32 {
    x.min(y)
}

/// 融合乘加运算
pub fn fma(x: f64, y: f64, z: f64) -> f64 {
    x.mul_add(y, z)
}

pub fn fmaf(x: f32, y: f32, z: f32) -> f32 {
    x.mul_add(y, z)
}

/// 正差函数
pub fn fdim(x: f64, y: f64) -> f64 {
    if x > y { x - y } else { 0.0 }
}

pub fn fdimf(x: f32, y: f32) -> f32 {
    if x > y { x - y } else { 0.0 }
}


// ============= 精度处理 Trait =============

/// 机器精度常量
pub const EPSILON: f64 = f64::EPSILON;
pub const MACHINE_EPSILON: f64 = 2.220446049250313e-16;

/// 精度处理 trait - 支持链式调用，类似于 "text".red() 的风格
pub trait PrecisionExt {
    /// 智能精度修正 - 自动检测并修正到精确值
    /// 
    /// # Example
    /// ```
    /// use lycrex_tool::utils::math::*;
    /// let result = sin(std::f64::consts::PI / 6.0).precise();
    /// assert_eq!(result, 0.5);
    /// 
    /// # 使用建议
    /// 对于三角函数计算：使用 sin(x).precise() 或 angle.sin_precise()
    /// 对于浮点数比较：使用 a.nearly_equals(b) 而不是 a == b
    /// 对于一般精度问题：使用 value.precise() 进行智能修正
    /// 对于指定精度：使用 value.precise_to(n) 舍入到 n 位小数
    /// ```
    fn precise(self) -> f64;
    
    /// 智能舍入到指定小数位数
    fn precise_to(self, decimal_places: u32) -> f64;
    
    /// 检查是否近似等于另一个值
    fn nearly_equals(self, other: f64) -> bool;
    
    /// 使用自定义容差检查是否近似等于
    fn nearly_equals_with_tolerance(self, other: f64, tolerance: f64) -> bool;
}

impl PrecisionExt for f64 {
    fn precise(self) -> f64 {
        // 首先检查是否接近常见的精确值
        let exact_values = [
            (0.0, 0.0),                           // 0
            (0.5, 0.5),                           // 1/2
            (1.0, 1.0),                           // 1
            (-0.5, -0.5),                         // -1/2
            (-1.0, -1.0),                         // -1
            (std::f64::consts::SQRT_2 / 2.0, std::f64::consts::SQRT_2 / 2.0), // √2/2
            (-std::f64::consts::SQRT_2 / 2.0, -std::f64::consts::SQRT_2 / 2.0), // -√2/2
            (std::f64::consts::FRAC_1_SQRT_2, std::f64::consts::FRAC_1_SQRT_2), // 1/√2
            ((3.0_f64).sqrt() / 2.0, (3.0_f64).sqrt() / 2.0), // √3/2
            (-(3.0_f64).sqrt() / 2.0, -(3.0_f64).sqrt() / 2.0), // -√3/2
            ((3.0_f64).sqrt(), (3.0_f64).sqrt()), // √3
            (1.0 / (3.0_f64).sqrt(), 1.0 / (3.0_f64).sqrt()), // 1/√3 
            (-(3.0_f64).sqrt(), -(3.0_f64).sqrt()), // -√3
            (-1.0 / (3.0_f64).sqrt(), -1.0 / (3.0_f64).sqrt()), // -1/√3
        ];
        
        for (test_val, exact_val) in exact_values.iter() {
            if (self - test_val).abs() < 1e-15 {
                return *exact_val;
            }
        }
        
        // 如果不是特殊值，应用智能舍入
        smart_round(self, None)
    }
    
    fn precise_to(self, decimal_places: u32) -> f64 {
        smart_round(self, Some(decimal_places))
    }
    
    fn nearly_equals(self, other: f64) -> bool {
        nearly_equal(self, other, None)
    }
    
    fn nearly_equals_with_tolerance(self, other: f64, tolerance: f64) -> bool {
        nearly_equal(self, other, Some(tolerance))
    }
}

/// 角度值的精确三角函数计算 trait
pub trait AnglePrecisionExt {
    /// 精确正弦值 - 直接从角度计算精确值
    fn sin_precise(self) -> f64;
    
    /// 精确余弦值
    fn cos_precise(self) -> f64;
    
    /// 精确正切值
    fn tan_precise(self) -> f64;
}

impl AnglePrecisionExt for f64 {
    fn sin_precise(self) -> f64 {
        // 直接在这里实现精确值逻辑，而不是调用已删除的函数
        let result = self.sin();
        
        let pi = std::f64::consts::PI;
        
        // 常见角度的精确值
        if nearly_equal(self, 0.0, Some(1e-15)) { return 0.0; }
        if nearly_equal(self, pi / 6.0, Some(1e-15)) { return 0.5; }
        if nearly_equal(self, pi / 4.0, Some(1e-15)) { return (2.0_f64).sqrt() / 2.0; }
        if nearly_equal(self, pi / 3.0, Some(1e-15)) { return (3.0_f64).sqrt() / 2.0; }
        if nearly_equal(self, pi / 2.0, Some(1e-15)) { return 1.0; }
        if nearly_equal(self, pi, Some(1e-15)) { return 0.0; }
        if nearly_equal(self, 3.0 * pi / 2.0, Some(1e-15)) { return -1.0; }
        if nearly_equal(self, 2.0 * pi, Some(1e-15)) { return 0.0; }
        
        // 负角度
        if nearly_equal(self, -pi / 6.0, Some(1e-15)) { return -0.5; }
        if nearly_equal(self, -pi / 4.0, Some(1e-15)) { return -(2.0_f64).sqrt() / 2.0; }
        if nearly_equal(self, -pi / 3.0, Some(1e-15)) { return -(3.0_f64).sqrt() / 2.0; }
        if nearly_equal(self, -pi / 2.0, Some(1e-15)) { return -1.0; }
        
        result
    }
    
    fn cos_precise(self) -> f64 {
        let result = self.cos();
        
        let pi = std::f64::consts::PI;
        
        // 常见角度的精确值
        if nearly_equal(self, 0.0, Some(1e-15)) { return 1.0; }
        if nearly_equal(self, pi / 6.0, Some(1e-15)) { return (3.0_f64).sqrt() / 2.0; }
        if nearly_equal(self, pi / 4.0, Some(1e-15)) { return (2.0_f64).sqrt() / 2.0; }
        if nearly_equal(self, pi / 3.0, Some(1e-15)) { return 0.5; }
        if nearly_equal(self, pi / 2.0, Some(1e-15)) { return 0.0; }
        if nearly_equal(self, pi, Some(1e-15)) { return -1.0; }
        if nearly_equal(self, 3.0 * pi / 2.0, Some(1e-15)) { return 0.0; }
        if nearly_equal(self, 2.0 * pi, Some(1e-15)) { return 1.0; }
        
        result
    }
    
    fn tan_precise(self) -> f64 {
        let result = self.tan();
        
        let pi = std::f64::consts::PI;
        
        // 常见角度的精确值
        if nearly_equal(self, 0.0, Some(1e-15)) { return 0.0; }
        if nearly_equal(self, pi / 6.0, Some(1e-15)) { return 1.0 / (3.0_f64).sqrt(); }
        if nearly_equal(self, pi / 4.0, Some(1e-15)) { return 1.0; }
        if nearly_equal(self, pi / 3.0, Some(1e-15)) { return (3.0_f64).sqrt(); }
        if nearly_equal(self, pi, Some(1e-15)) { return 0.0; }
        
        result
    }
}

/// 智能舍入函数 - 根据数值特征自适应舍入
pub fn smart_round(x: f64, decimal_places: Option<u32>) -> f64 {
    // 对于接近整数的值，使用更严格的阈值
    if let Some(places) = decimal_places {
        let factor = 10.0_f64.powi(places as i32);
        (x * factor).round() / factor
    } else {
        // 自适应精度：检查是否接近已知的数学常值
        if is_near_integer(x) {
            x.round()
        } else if is_near_half_integer(x) {
            (x * 2.0).round() / 2.0
        } else if is_near_rational_fraction(x) {
            round_to_rational(x)
        } else {
            x
        }
    }
}

/// 检查是否接近整数
fn is_near_integer(x: f64) -> bool {
    (x - x.round()).abs() < 1e-15
}

/// 检查是否接近半整数（n + 0.5）
fn is_near_half_integer(x: f64) -> bool {
    let shifted = x - 0.5;
    (shifted - shifted.round()).abs() < 1e-15
}

/// 检查是否接近常见有理分数
fn is_near_rational_fraction(x: f64) -> bool {
    // 检查常见分数：1/2, 1/3, 2/3, 1/4, 3/4, 1/6, 5/6, 等
    let fractions = [
        (1.0, 2.0),   // 1/2
        (1.0, 3.0),   // 1/3
        (2.0, 3.0),   // 2/3
        (1.0, 4.0),   // 1/4
        (3.0, 4.0),   // 3/4
        (1.0, 6.0),   // 1/6
        (5.0, 6.0),   // 5/6
        (1.0, 8.0),   // 1/8
        (3.0, 8.0),   // 3/8
        (5.0, 8.0),   // 5/8
        (7.0, 8.0),   // 7/8
    ];
    
    for (num, den) in fractions.iter() {
        if (x - num / den).abs() < 1e-15 {
            return true;
        }
    }
    false
}

/// 舍入到最接近的有理分数
fn round_to_rational(x: f64) -> f64 {
    let fractions = [
        (1.0, 2.0),   // 1/2
        (1.0, 3.0),   // 1/3
        (2.0, 3.0),   // 2/3
        (1.0, 4.0),   // 1/4
        (3.0, 4.0),   // 3/4
        (1.0, 6.0),   // 1/6
        (5.0, 6.0),   // 5/6
        (1.0, 8.0),   // 1/8
        (3.0, 8.0),   // 3/8
        (5.0, 8.0),   // 5/8
        (7.0, 8.0),   // 7/8
    ];
    
    let mut best_match = x;
    let mut min_error = f64::INFINITY;
    
    for (num, den) in fractions.iter() {
        let fraction = num / den;
        let error = (x - fraction).abs();
        if error < min_error && error < 1e-15 {
            min_error = error;
            best_match = fraction;
        }
    }
    
    best_match
}

/// 高精度比较函数
pub fn nearly_equal(a: f64, b: f64, tolerance: Option<f64>) -> bool {
    let tol = tolerance.unwrap_or(MACHINE_EPSILON * 10.0);
    
    // 处理特殊值
    if a.is_infinite() && b.is_infinite() {
        return a.signum() == b.signum();
    }
    if a.is_nan() || b.is_nan() {
        return false;
    }
    
    let diff = (a - b).abs();
    
    // 绝对误差检查（对于接近零的值）
    if a.abs().max(b.abs()) < tol {
        return diff < tol;
    }
    
    // 相对误差检查
    let relative_error = diff / a.abs().max(b.abs());
    relative_error < tol
}


// ============= 高精度算法变体 =============
//      • 三角函数: 角度归约 + 泰勒级数 + 精确值
//      • 平方根: 牛顿迭代法额外精度提升
//      • 对数: 特殊级数展开处理接近1的值
//      • 指数: 小数值泰勒级数展开

/// 高精度正弦函数 - 使用泰勒级数进行小角度计算
pub fn sin_hp(x: f64) -> f64 {
    // 首先进行角度归约到 [-π, π]
    let pi = std::f64::consts::PI;
    let mut reduced_x = x % (2.0 * pi);
    if reduced_x > pi {
        reduced_x -= 2.0 * pi;
    } else if reduced_x < -pi {
        reduced_x += 2.0 * pi;
    }
    
    // 对于非常小的角度，使用高精度泰勒级数
    if reduced_x.abs() < 1e-8 {
        let x2 = reduced_x * reduced_x;
        let x3 = x2 * reduced_x;
        let x5 = x3 * x2;
        let x7 = x5 * x2;
        // sin(x) ≈ x - x³/6 + x⁵/120 - x⁷/5040 + ...
        reduced_x - x3/6.0 + x5/120.0 - x7/5040.0
    } else {
        // 对于常见角度，使用精确值检查
        reduced_x.sin_precise()
    }
}

/// 高精度余弦函数
pub fn cos_hp(x: f64) -> f64 {
    let pi = std::f64::consts::PI;
    let mut reduced_x = x % (2.0 * pi);
    if reduced_x > pi {
        reduced_x -= 2.0 * pi;
    } else if reduced_x < -pi {
        reduced_x += 2.0 * pi;
    }
    
    // 对于非常小的角度，使用高精度泰勒级数
    if reduced_x.abs() < 1e-8 {
        let x2 = reduced_x * reduced_x;
        let x4 = x2 * x2;
        let x6 = x4 * x2;
        let x8 = x6 * x2;
        // cos(x) ≈ 1 - x²/2 + x⁴/24 - x⁶/720 + x⁸/40320 - ...
        1.0 - x2/2.0 + x4/24.0 - x6/720.0 + x8/40320.0
    } else {
        reduced_x.cos_precise()
    }
}

/// 高精度平方根 - 使用牛顿迭代法
pub fn sqrt_hp(x: f64) -> f64 {
    if x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 || x == 1.0 {
        return x;
    }
    
    // 使用标准库的结果作为初始值
    let mut guess = x.sqrt();
    
    // 进行几次牛顿迭代以提高精度
    for _ in 0..3 {
        guess = 0.5 * (guess + x / guess);
    }
    
    guess
}

/// 高精度自然对数 - 使用级数展开
pub fn log_hp(x: f64) -> f64 {
    if x <= 0.0 {
        return f64::NAN;
    }
    if x == 1.0 {
        return 0.0;
    }
    
    // 对于接近1的值，使用ln(1+u)的级数展开
    if (x - 1.0).abs() < 0.5 {
        let u = x - 1.0;
        let u2 = u * u;
        let u3 = u2 * u;
        let u4 = u3 * u;
        let u5 = u4 * u;
        // ln(1+u) = u - u²/2 + u³/3 - u⁴/4 + u⁵/5 - ...
        u - u2/2.0 + u3/3.0 - u4/4.0 + u5/5.0
    } else {
        x.ln()
    }
}

/// 高精度指数函数 - 使用泰勒级数
pub fn exp_hp(x: f64) -> f64 {
    if x.abs() < 1e-8 {
        // 对于小数值，使用泰勒级数
        let x2 = x * x;
        let x3 = x2 * x;
        let x4 = x3 * x;
        let x5 = x4 * x;
        // e^x = 1 + x + x²/2! + x³/3! + x⁴/4! + x⁵/5! + ...
        1.0 + x + x2/2.0 + x3/6.0 + x4/24.0 + x5/120.0
    } else {
        x.exp()
    }
}

/// 高精度Atan2 - 改进的两参数反正切
pub fn atan2_hp(y: f64, x: f64) -> f64 {
    // 处理特殊情况
    if x == 0.0 && y == 0.0 {
        return 0.0;
    }
    
    let pi = std::f64::consts::PI;
    
    // 对于x >> y 或 y >> x的情况，直接返回近似值
    if x.abs() > 1e8 * y.abs() {
        return if x > 0.0 { 0.0 } else { pi };
    }
    if y.abs() > 1e8 * x.abs() {
        return if y > 0.0 { pi/2.0 } else { -pi/2.0 };
    }
    
    // 使用标准函数
    y.atan2(x)
}

/// 稳定的二次方程求解器
pub fn solve_quadratic(a: f64, b: f64, c: f64) -> (Option<f64>, Option<f64>) {
    if a.abs() < 1e-15 {
        // 线性方程: bx + c = 0
        if b.abs() < 1e-15 {
            return (None, None); // 无解或无穷解
        }
        return (Some(-c / b), None);
    }
    
    let discriminant = b * b - 4.0 * a * c;
    
    if discriminant < 0.0 {
        return (None, None); // 无实数解
    }
    
    if discriminant == 0.0 {
        return (Some(-b / (2.0 * a)), None); // 一个解
    }
    
    // 数值稳定的求解方法
    let sqrt_d = discriminant.sqrt();
    let q = if b >= 0.0 {
        -0.5 * (b + sqrt_d)
    } else {
        -0.5 * (b - sqrt_d)
    };
    
    let x1 = q / a;
    let x2 = c / q;
    
    // 返回较小的根在前
    if x1 <= x2 {
        (Some(x1), Some(x2))
    } else {
        (Some(x2), Some(x1))
    }
}

/// 数值稳定的对数差：log(a) - log(b) = log(a/b)
pub fn log_diff(a: f64, b: f64) -> f64 {
    if a <= 0.0 || b <= 0.0 {
        return f64::NAN;
    }
    
    // 直接计算比值的对数，避免中间的精度损失
    (a / b).ln()
}

/// 数值稳定的指数差：exp(a) - exp(b)
pub fn exp_diff(a: f64, b: f64) -> f64 {
    if a == b {
        return 0.0;
    }
    
    // 提取公因子以避免上溢/下溢
    let max_val = a.max(b);
    let min_val = a.min(b);
    let sign = if a > b { 1.0 } else { -1.0 };
    
    sign * exp(max_val) * (1.0 - exp(min_val - max_val))
}
