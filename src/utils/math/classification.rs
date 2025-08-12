/// 浮点数分类和检测函数

use super::constants::*;

/// 浮点数分类函数
pub fn fpclassify(x: f64) -> i32 {
    if x.is_nan() {
        FP_NAN
    } else if x.is_infinite() {
        FP_INFINITE
    } else if x == 0.0 {
        FP_ZERO
    } else if x.is_subnormal() {
        FP_SUBNORMAL
    } else {
        FP_NORMAL
    }
}

pub fn fpclassifyf(x: f32) -> i32 {
    if x.is_nan() {
        FP_NAN
    } else if x.is_infinite() {
        FP_INFINITE
    } else if x == 0.0 {
        FP_ZERO
    } else if x.is_subnormal() {
        FP_SUBNORMAL
    } else {
        FP_NORMAL
    }
}

/// 检测是否为有限数
pub fn isfinite(x: f64) -> bool {
    x.is_finite()
}

pub fn isfinitef(x: f32) -> bool {
    x.is_finite()
}

/// 检测是否为无穷大
pub fn isinf(x: f64) -> bool {
    x.is_infinite()
}

pub fn isinff(x: f32) -> bool {
    x.is_infinite()
}

/// 检测是否为NaN
pub fn isnan(x: f64) -> bool {
    x.is_nan()
}

pub fn isnanf(x: f32) -> bool {
    x.is_nan()
}

/// 检测是否为正常数（非零、非无穷、非NaN、非次正常数）
pub fn isnormal(x: f64) -> bool {
    x.is_normal()
}

pub fn isnormalf(x: f32) -> bool {
    x.is_normal()
}

/// 检测符号位
pub fn signbit(x: f64) -> bool {
    x.is_sign_negative()
}

pub fn signbitf(x: f32) -> bool {
    x.is_sign_negative()
}

/// 获取指数
pub fn ilogb(x: f64) -> i32 {
    if x == 0.0 {
        FP_ILOGB0
    } else if x.is_nan() {
        FP_ILOGBNAN
    } else if x.is_infinite() {
        i32::MAX
    } else {
        let (_, exp) = super::basic::frexp(x.abs());
        exp - 1
    }
}

pub fn ilogbf(x: f32) -> i32 {
    if x == 0.0 {
        FP_ILOGB0
    } else if x.is_nan() {
        FP_ILOGBNAN
    } else if x.is_infinite() {
        i32::MAX
    } else {
        let (_, exp) = super::basic::frexpf(x.abs());
        exp - 1
    }
}

/// 获取指数部分（作为浮点数）
pub fn logb(x: f64) -> f64 {
    if x == 0.0 {
        f64::NEG_INFINITY
    } else if x.is_nan() {
        f64::NAN
    } else if x.is_infinite() {
        f64::INFINITY
    } else {
        ilogb(x) as f64
    }
}

pub fn logbf(x: f32) -> f32 {
    if x == 0.0 {
        f32::NEG_INFINITY
    } else if x.is_nan() {
        f32::NAN
    } else if x.is_infinite() {
        f32::INFINITY
    } else {
        ilogbf(x) as f32
    }
}

/// 缩放函数
pub fn scalbn(x: f64, n: i32) -> f64 {
    super::basic::ldexp(x, n)
}

pub fn scalbnf(x: f32, n: i32) -> f32 {
    super::basic::ldexpf(x, n)
}

pub fn scalbln(x: f64, n: i64) -> f64 {
    super::basic::ldexp(x, n as i32)
}

pub fn scalblnf(x: f32, n: i64) -> f32 {
    super::basic::ldexpf(x, n as i32)
}

/// 创建NaN
pub fn nan(_tagp: &str) -> f64 {
    // 在实际实现中，tagp可以用于创建特定的NaN值
    // 这里简化处理，直接返回标准NaN
    f64::NAN
}

pub fn nanf(_tagp: &str) -> f32 {
    f32::NAN
}

/// 下一个可表示的值
pub fn nextafter(x: f64, y: f64) -> f64 {
    if x == y {
        return y;
    }
    
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }
    
    if x == 0.0 {
        return if y > 0.0 { f64::MIN_POSITIVE } else { -f64::MIN_POSITIVE };
    }
    
    let bits = x.to_bits();
    let new_bits = if (x < y && x > 0.0) || (x > y && x < 0.0) {
        bits + 1
    } else {
        bits - 1
    };
    
    f64::from_bits(new_bits)
}

pub fn nextafterf(x: f32, y: f32) -> f32 {
    if x == y {
        return y;
    }
    
    if x.is_nan() || y.is_nan() {
        return f32::NAN;
    }
    
    if x == 0.0 {
        return if y > 0.0 { f32::MIN_POSITIVE } else { -f32::MIN_POSITIVE };
    }
    
    let bits = x.to_bits();
    let new_bits = if (x < y && x > 0.0) || (x > y && x < 0.0) {
        bits + 1
    } else {
        bits - 1
    };
    
    f32::from_bits(new_bits)
}

/// 舍入函数
pub fn rint(x: f64) -> f64 {
    // 使用当前舍入模式进行舍入
    x.round()
}

pub fn rintf(x: f32) -> f32 {
    x.round()
}

pub fn lrint(x: f64) -> i64 {
    x.round() as i64
}

pub fn lrintf(x: f32) -> i64 {
    x.round() as i64
}

pub fn llrint(x: f64) -> i64 {
    x.round() as i64
}

pub fn llrintf(x: f32) -> i64 {
    x.round() as i64
}

pub fn lround(x: f64) -> i64 {
    x.round() as i64
}

pub fn lroundf(x: f32) -> i64 {
    x.round() as i64
}

pub fn llround(x: f64) -> i64 {
    x.round() as i64
}

pub fn llroundf(x: f32) -> i64 {
    x.round() as i64
}

pub fn nearbyint(x: f64) -> f64 {
    x.round()
}

pub fn nearbyintf(x: f32) -> f32 {
    x.round()
}

pub fn isgreater(x: f64, y: f64) -> bool {
    !x.is_nan() && !y.is_nan() && x > y
}

pub fn isgreaterequal(x: f64, y: f64) -> bool {
    !x.is_nan() && !y.is_nan() && x >= y
}

pub fn isless(x: f64, y: f64) -> bool {
    !x.is_nan() && !y.is_nan() && x < y
}

pub fn islessequal(x: f64, y: f64) -> bool {
    !x.is_nan() && !y.is_nan() && x <= y
}

pub fn islessgreater(x: f64, y: f64) -> bool {
    !x.is_nan() && !y.is_nan() && (x < y || x > y)
}

pub fn isunordered(x: f64, y: f64) -> bool {
    x.is_nan() || y.is_nan()
}
