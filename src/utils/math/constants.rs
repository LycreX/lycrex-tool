/// 数学常量定义

/// 自然对数的底数 e
pub const M_E: f64 = 2.71828182845904523536;

/// log2(e)
pub const M_LOG2E: f64 = 1.44269504088896340736;

/// log10(e)
pub const M_LOG10E: f64 = 0.434294481903251827651;

/// ln(2)
pub const M_LN2: f64 = 0.693147180559945309417;

/// ln(10)
pub const M_LN10: f64 = 2.30258509299404568402;

/// 圆周率 π
pub const M_PI: f64 = 3.14159265358979323846;

/// π/2
pub const M_PI_2: f64 = 1.57079632679489661923;

/// π/4
pub const M_PI_4: f64 = 0.785398163397448309616;

/// 1/π
pub const M_1_PI: f64 = 0.318309886183790671538;

/// 2/π
pub const M_2_PI: f64 = 0.636619772367581343076;

/// 2/√π
pub const M_2_SQRTPI: f64 = 1.12837916709551257390;

/// √2
pub const M_SQRT2: f64 = 1.41421356237309504880;

/// 1/√2 = √2/2
pub const M_SQRT1_2: f64 = 0.707106781186547524401;

/// 错误域定义
pub const DOMAIN: i32 = 1;
pub const SING: i32 = 2;
pub const OVERFLOW: i32 = 3;
pub const UNDERFLOW: i32 = 4;
pub const TLOSS: i32 = 5;
pub const PLOSS: i32 = 6;

/// 浮点数分类常量
pub const FP_NAN: i32 = 0x0100;
pub const FP_NORMAL: i32 = 0x0400;
pub const FP_INFINITE: i32 = FP_NAN | FP_NORMAL;
pub const FP_ZERO: i32 = 0x4000;
pub const FP_SUBNORMAL: i32 = FP_NORMAL | FP_ZERO;

/// 特殊值
pub const NAN: f32 = f32::NAN;
pub const INFINITY: f32 = f32::INFINITY;
pub const HUGE_VALF: f32 = f32::INFINITY;
pub const HUGE_VALL: f64 = f64::INFINITY;

/// 浮点数舍入模式
pub const FE_TONEAREST: i32 = 0x0000;
pub const FE_DOWNWARD: i32 = 0x0400;
pub const FE_UPWARD: i32 = 0x0800;
pub const FE_TOWARDZERO: i32 = 0x0c00;

/// ilogb函数的特殊返回值
pub const FP_ILOGB0: i32 = i32::MIN;
pub const FP_ILOGBNAN: i32 = i32::MIN;
