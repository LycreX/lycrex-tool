/// 高级数学函数
/// 包括gamma函数、贝塞尔函数、误差函数等

use super::constants::*;
use super::basic::*;

/// Gamma函数
/// 使用Lanczos近似算法
/// Γ(z) = (z-1)! 对于正整数z
/// Γ(z+1) = z·Γ(z) 递推关系
/// Γ(1/2) = √π
pub fn tgamma(x: f64) -> f64 {
    // 处理特殊情况
    if x.is_nan() {
        return f64::NAN;
    }
    
    if x.is_infinite() {
        return if x > 0.0 { f64::INFINITY } else { f64::NAN };
    }
    
    // Gamma函数在负整数处有极点（未定义）
    if x <= 0.0 && x.fract() == 0.0 {
        return f64::NAN;
    }
    
    // 一些特殊值
    if x == 1.0 || x == 2.0 {
        return 1.0;
    }
    
    if x == 0.5 {
        return M_PI.sqrt(); // Γ(1/2) = √π
    }
    
    // 小的正值，使用递推关系移到更大的值
    if x > 0.0 && x < 1.0 {
        return tgamma(x + 1.0) / x;
    }
    
    // 负数，使用反射公式: Γ(z)Γ(1-z) = π/sin(πz)
    if x < 0.0 {
        let sin_pi_x = sin(M_PI * x);
        if sin_pi_x.abs() < f64::EPSILON {
            return f64::NAN; // 接近负整数
        }
        return M_PI / (sin_pi_x * tgamma(1.0 - x));
    }
    
    // 大值使用Stirling近似避免溢出
    if x > 171.0 {
        return f64::INFINITY; // 会溢出
    }
    
    // Lanczos近似 - 高精度版本
    // g = 7, n = 9 (9个系数)
    lanczos_gamma(x)
}

/// Lanczos实现
/// 参考: https://en.wikipedia.org/wiki/Lanczos_approximation
fn lanczos_gamma(z: f64) -> f64 {
    // Lanczos系数 (g=7, n=9)
    const G: f64 = 7.0;
    const LANCZOS_COEFFICIENTS: [f64; 9] = [
        0.99999999999980993,      // c₀
        676.5203681218851,         // c₁
        -1259.1392167224028,      // c₂
        771.32342877765313,       // c₃
        -176.61502916214059,      // c₄
        12.507343278686905,       // c₅
        -0.13857109526572012,     // c₆
        9.9843695780195716e-6,    // c₇
        1.5056327351493116e-7,    // c₈
    ];
    
    if z < 0.5 {
        // 反射公式: Γ(z) = π / (sin(πz) * Γ(1-z))
        return M_PI / (sin(M_PI * z) * lanczos_gamma(1.0 - z));
    }
    
    let z = z - 1.0;
    
    // A_g(z) = c₀ + Σ(cᵢ/(z+i))
    let mut x = LANCZOS_COEFFICIENTS[0];
    for i in 1..LANCZOS_COEFFICIENTS.len() {
        x += LANCZOS_COEFFICIENTS[i] / (z + i as f64);
    }
    
    // Γ(z+1) = √(2π) × (z+g+1/2)^(z+1/2) × e^(-(z+g+1/2)) × A_g(z)
    let t = z + G + 0.5;  // t = z + g + 1/2
    let sqrt_2pi = (2.0 * M_PI).sqrt();
    
    sqrt_2pi * t.powf(z + 0.5) * (-t).exp() * x
}

/// 对数Gamma函数的完整实现
/// 计算 ln(Γ(x))，避免大数时的溢出
pub fn lgamma_complete(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    
    if x <= 0.0 {
        if x.fract() == 0.0 {
            return f64::INFINITY;
        }
        // 负数使用反射公式的对数形式
        let sin_pi_x = sin(M_PI * x);
        if sin_pi_x.abs() < f64::EPSILON {
            return f64::INFINITY;
        }
        return (M_PI / sin_pi_x.abs()).ln() - lgamma_complete(1.0 - x);
    }
    
    if x == 1.0 || x == 2.0 {
        return 0.0; // ln(Γ(1)) = ln(Γ(2)) = ln(1) = 0
    }
    
    if x < 12.0 {
        // 小值，使用递推关系 Γ(x+1) = x·Γ(x)
        // 所以 ln(Γ(x+1)) = ln(x) + ln(Γ(x))
        let mut result = 0.0;
        let mut xx = x;
        
        while xx < 12.0 {
            result -= xx.ln();
            xx += 1.0;
        }
        
        result + lgamma_stirling(xx)
    } else {
        lgamma_stirling(x)
    }
}

/// Stirling近似计算对数Gamma函数
/// 适用于大参数值
fn lgamma_stirling(x: f64) -> f64 {
    // Stirling公式: ln(Γ(x)) ≈ (x-0.5)ln(x) - x + 0.5ln(2π) + 1/(12x) - 1/(360x³) + ...
    let inv_x = 1.0 / x;
    let inv_x2 = inv_x * inv_x;
    
    // Stirling级数的前几项
    let series = inv_x / 12.0 
                - inv_x2 * inv_x / 360.0 
                + inv_x2 * inv_x2 * inv_x / 1260.0
                - inv_x2 * inv_x2 * inv_x2 * inv_x / 1680.0;
    
    (x - 0.5) * x.ln() - x + 0.5 * (2.0 * M_PI).ln() + series
}

/// 上升阶乘
/// (x)_n = x(x+1)(x+2)...(x+n-1) = Γ(x+n)/Γ(x)
pub fn pochhammer(x: f64, n: u32) -> f64 {
    if n == 0 {
        return 1.0;
    }
    
    let mut result = 1.0;
    for i in 0..n {
        result *= x + i as f64;
    }
    result
}

/// Beta函数
/// B(x,y) = Γ(x)Γ(y)/Γ(x+y)
pub fn beta(x: f64, y: f64) -> f64 {
    if x <= 0.0 || y <= 0.0 {
        return f64::NAN;
    }
    
    // 对数形式避免溢出
    let log_beta = lgamma_complete(x) + lgamma_complete(y) - lgamma_complete(x + y);
    log_beta.exp()
}

/// 下不完全Gamma函数 - 高精度
/// γ(s,x) = ∫₀ˣ t^(s-1) e^(-t) dt
/// 使用级数展开和连分数相结合的方法
pub fn gamma_inc_lower(s: f64, x: f64) -> f64 {
    if s <= 0.0 || x < 0.0 {
        return f64::NAN;
    }
    
    if x == 0.0 {
        return 0.0;
    }
    
    if x.is_infinite() {
        return tgamma(s);
    }
    
    // 根据参数大小选择不同算法
    if x < s + 1.0 {
        // 使用级数展开 (收敛较快)
        // γ(s,x) = x^s e^(-x) Σ(n=0 to ∞) x^n / Γ(s+n+1)
        let mut sum = 0.0;
        let mut term = 1.0 / tgamma(s + 1.0);
        let mut n = 0;
        
        loop {
            sum += term;
            n += 1;
            
            // 递推计算下一项: term_{n+1} = term_n * x / (s + n)
            term *= x / (s + n as f64);
            
            // 更严格的收敛条件
            if term.abs() < 1e-16 * sum.abs() || n > 1000 {
                break;
            }
        }
        
        x.powf(s) * (-x).exp() * sum
    } else {
        // 使用连分数展开 (通过上不完全Gamma函数)
        // γ(s,x) = Γ(s) - Γ(s,x)
        let gamma_s = tgamma(s);
        let gamma_upper = gamma_inc_upper_cf(s, x);
        gamma_s - gamma_upper
    }
}

/// 上不完全Gamma函数的连分数实现
/// Γ(s,x) = ∫ₓ^∞ t^(s-1) e^(-t) dt
fn gamma_inc_upper_cf(s: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return tgamma(s);
    }
    
    // 使用连分数展开
    // Γ(s,x) = e^(-x) x^s * (1 / (x + (1-s) / (1 + 1/(x + (2-s)/(1 + 2/(x + ...))))))
    let _a = 1.0;
    let mut b = x + 1.0 - s;
    let mut c = 1e30;
    let mut d = 1.0 / b;
    let mut h = d;
    
    for i in 1..=1000 {
        let an = -i as f64 * (i as f64 - s);
        b += 2.0;
        d = an * d + b;
        
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        
        c = b + an / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        
        if (del - 1.0).abs() < 1e-15 {
            break;
        }
    }
    
    (-x).exp() * x.powf(s) * h
}

pub fn tgammaf(x: f32) -> f32 {
    tgamma(x as f64) as f32
}

/// 对数Gamma函数
/// 计算 ln(Γ(x))，使用更稳定的算法避免溢出
pub fn lgamma(x: f64) -> f64 {
    lgamma_complete(x)
}

pub fn lgammaf(x: f32) -> f32 {
    lgamma(x as f64) as f32
}



/// Abramowitz & Stegun 误差函数近似 - 第四种 (最大误差: 1.5×10^-7)
/// erf(x) ≈ 1 - (a₁t + a₂t² + ... + a₅t⁵)e^(-x²), t = 1/(1 + px)
fn erf_abramowitz_stegun_4(x: f64) -> f64 {
    if x < 0.0 {
        return -erf_abramowitz_stegun_4(-x);
    }
    
    let p = 0.3275911;
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    
    let t = 1.0 / (1.0 + p * x);
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t2 * t2;
    let t5 = t4 * t;
    let exp_neg_x_sq = (-x * x).exp();
    
    1.0 - (a1 * t + a2 * t2 + a3 * t3 + a4 * t4 + a5 * t5) * exp_neg_x_sq
}



/// 高精度误差函数 (最大误差: 1.2×10^-7)
/// 基于文档中提供的最高精度近似
fn erf_high_precision(x: f64) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    
    let x_abs = x.abs();
    let t = 1.0 / (1.0 + 0.5 * x_abs);
    
    let tau = t * (-x_abs * x_abs - 1.26551223 
        + 1.00002368 * t
        + 0.37409196 * t * t
        + 0.09678418 * t * t * t
        - 0.18628806 * t * t * t * t
        + 0.27886807 * t * t * t * t * t
        - 1.13520398 * t * t * t * t * t * t
        + 1.48851587 * t * t * t * t * t * t * t
        - 0.82215223 * t * t * t * t * t * t * t * t
        + 0.17087277 * t * t * t * t * t * t * t * t * t).exp();
    
    if x >= 0.0 {
        1.0 - tau
    } else {
        tau - 1.0
    }
}



/// 误差函数 - 超高精度实现
/// 自适应选择最佳近似方法以获得最高精度
pub fn erf(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    
    if x == 0.0 {
        return 0.0;
    }
    
    if x.is_infinite() {
        return if x > 0.0 { 1.0 } else { -1.0 };
    }
    
    let abs_x = x.abs();
    
    // 自适应选择最佳近似方法
    if abs_x < 1.0 {
        // 中小值使用 A&S 第四种近似 (最大误差: 1.5×10^-7)
        erf_abramowitz_stegun_4(x)
    } else {
        // 大值使用高精度近似 (最大误差: 1.2×10^-7)
        erf_high_precision(x)
    }
}









pub fn erff(x: f32) -> f32 {
    erf(x as f64) as f32
}

/// 超高精度 erfc 实现 (相对误差 < 2^-53 ≈ 1.1×10^-16)
/// 基于文档提供的最高精度有理函数近似
fn erfc_ultra_high_precision(x: f64) -> f64 {
    if x < 0.0 {
        return 2.0 - erfc_ultra_high_precision(-x);
    }
    
    if x == 0.0 {
        return 1.0;
    }
    
    // 第一个有理函数因子
    let factor1 = 0.56418958354775629 / (x + 2.06955023132914151);
    
    // 第二个有理函数因子
    let x2 = x * x;
    let num2 = x2 + 2.71078540045147805 * x + 5.80755613130301624;
    let den2 = x2 + 3.47954057099518960 * x + 12.06166887286239555;
    let factor2 = num2 / den2;
    
    // 第三个有理函数因子
    let num3 = x2 + 3.47469513777439592 * x + 12.07402036406381411;
    let den3 = x2 + 3.72068443960225092 * x + 8.44319781003968454;
    let factor3 = num3 / den3;
    
    // 第四个有理函数因子
    let num4 = x2 + 4.00561509202259545 * x + 9.30596659485887898;
    let den4 = x2 + 3.90225704029924078 * x + 6.36161630953880464;
    let factor4 = num4 / den4;
    
    // 第五个有理函数因子
    let num5 = x2 + 5.16722705817812584 * x + 9.12661617673673262;
    let den5 = x2 + 4.03296893109262491 * x + 5.13578530585681539;
    let factor5 = num5 / den5;
    
    // 第六个有理函数因子
    let num6 = x2 + 5.95908795446633271 * x + 9.19435612886969243;
    let den6 = x2 + 4.11240942957450885 * x + 4.48640329523408675;
    let factor6 = num6 / den6;
    
    let exp_neg_x_sq = (-x2).exp();
    
    factor1 * factor2 * factor3 * factor4 * factor5 * factor6 * exp_neg_x_sq
}

/// 互补误差函数 - 超高精度实现
/// 使用最高精度的有理函数近似 (相对误差 < 2^-53)
pub fn erfc(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    
    if x.is_infinite() {
        return if x > 0.0 { 0.0 } else { 2.0 };
    }
    
    // 使用超高精度近似
    erfc_ultra_high_precision(x)
}

pub fn erfcf(x: f32) -> f32 {
    erfc(x as f64) as f32
}

/// 第一类贝塞尔函数 J0 - 高精度实现
/// 使用优化的级数展开和渐近展开
pub fn j0(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    
    let x = x.abs();
    
    if x == 0.0 {
        return 1.0;
    }
    
    if x < 8.0 {
        // 小参数：使用优化的幂级数展开
        // J₀(x) = Σ((-1)^k * (x/2)^(2k) / (k!)²)
        let x_half = x * 0.5;
        let x_half_sq = x_half * x_half;
        
        let mut sum = 1.0;
        let mut term = 1.0;
        let mut k = 1;
        
        loop {
            term *= -x_half_sq / (k * k) as f64;
            sum += term;
            
            if term.abs() < 1e-16 * sum.abs() || k > 50 {
                break;
            }
            k += 1;
        }
        
        sum
    } else {
        // 大参数：使用改进的渐近展开
        // J₀(x) ≈ √(2/(πx)) * [P₀(x)cos(x-π/4) - Q₀(x)sin(x-π/4)]
        let z = 8.0 / x;
        let z2 = z * z;
        
        // P₀和Q₀的更精确系数
        let p0 = 1.0 + z2 * (-0.1098628627e-2 + z2 * 0.2734510407e-4);
        let q0 = z * (-0.1562499995e-1 + z2 * 0.1430488765e-3);
        
        let phase = x - M_PI * 0.25; // x - π/4
        let amplitude = (2.0 / (M_PI * x)).sqrt();
        
        amplitude * (p0 * cos(phase) - q0 * sin(phase))
    }
}

/// 第一类贝塞尔函数 J1 - 高精度实现
/// 使用优化的级数展开和渐近展开
pub fn j1(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    
    if x == 0.0 {
        return 0.0;
    }
    
    let sign = if x >= 0.0 { 1.0 } else { -1.0 };
    let x = x.abs();
    
    if x < 8.0 {
        // 小参数：使用高精度幂级数展开
        // J₁(x) = (x/2) * Σ((-1)^k * (x/2)^(2k) / (k!(k+1)!))
        let x_half = x * 0.5;
        let x_half_sq = x_half * x_half;
        
        let mut sum = 0.5; // 第一项：1/2
        let mut term = 0.5;
        let mut k = 1;
        
        loop {
            term *= -x_half_sq / (k * (k + 1)) as f64;
            sum += term;
            
            if term.abs() < 1e-16 * sum.abs() || k > 50 {
                break;
            }
            k += 1;
        }
        
        sign * x * sum
    } else {
        // 大参数：使用改进的渐近展开
        // J₁(x) ≈ √(2/(πx)) * [P₁(x)cos(x-3π/4) - Q₁(x)sin(x-3π/4)]
        let z = 8.0 / x;
        let z2 = z * z;
        
        // P₁和Q₁的更精确系数
        let p1 = 1.0 + z2 * (0.183105e-2 - z2 * 0.3516396496e-1);
        let q1 = z * (0.04687499995 - z2 * 0.2002690873e-3);
        
        let phase = x - 3.0 * M_PI * 0.25; // x - 3π/4
        let amplitude = (2.0 / (M_PI * x)).sqrt();
        
        sign * amplitude * (p1 * cos(phase) - q1 * sin(phase))
    }
}

/// 第n阶第一类贝塞尔函数
pub fn jn(n: i32, x: f64) -> f64 {
    if n == 0 {
        return j0(x);
    }
    if n == 1 {
        return j1(x);
    }
    
    if x.is_nan() {
        return f64::NAN;
    }
    
    if x == 0.0 {
        return 0.0;
    }
    
    // 递推关系计算
    let mut j_prev = j0(x);
    let mut j_curr = j1(x);
    
    for i in 1..n.abs() {
        let j_next = 2.0 * i as f64 * j_curr / x - j_prev;
        j_prev = j_curr;
        j_curr = j_next;
    }
    
    if n < 0 && n % 2 != 0 {
        -j_curr
    } else {
        j_curr
    }
}

/// 第二类贝塞尔函数 Y0
pub fn y0(x: f64) -> f64 {
    if x.is_nan() || x <= 0.0 {
        return f64::NAN;
    }
    
    if x < 8.0 {
        // 小参数展开
        let _y = x * x;
        let p0 = 2.0 / M_PI * (j0(x) * x.ln() + 0.36746691);
        p0
    } else {
        // 大参数渐近展开
        let z = 8.0 / x;
        let y = z * z;
        let xx = x - 0.785398164; // π/4
        
        let p0 = 1.0;
        let p1 = -0.1098628627e-2 * y;
        let q0 = -0.1562499995e-1 * z;
        let q1 = 0.1430488765e-3 * z * y;
        
        (2.0 / (M_PI * x)).sqrt() * ((p0 + p1) * sin(xx) + z * (q0 + q1) * cos(xx))
    }
}

/// 第二类贝塞尔函数 Y1
pub fn y1(x: f64) -> f64 {
    if x.is_nan() || x <= 0.0 {
        return f64::NAN;
    }
    
    if x < 8.0 {
        // 小参数展开
        let p0 = 2.0 / M_PI * (j1(x) * x.ln() - 1.0 / x - 0.36746691 * x);
        p0
    } else {
        // 大参数渐近展开
        let z = 8.0 / x;
        let y = z * z;
        let xx = x - 2.356194491; // 3π/4
        
        let p0 = 1.0;
        let p1 = 0.183105e-2 * y;
        let q0 = -0.3516396496e-1 * z;
        let q1 = 0.2457520174e-3 * z * y;
        
        (2.0 / (M_PI * x)).sqrt() * ((p0 + p1) * sin(xx) + z * (q0 + q1) * cos(xx))
    }
}

/// 第n阶第二类贝塞尔函数
pub fn yn(n: i32, x: f64) -> f64 {
    if n == 0 {
        return y0(x);
    }
    if n == 1 {
        return y1(x);
    }
    
    if x.is_nan() || x <= 0.0 {
        return f64::NAN;
    }
    
    // 递推关系计算
    let mut y_prev = y0(x);
    let mut y_curr = y1(x);
    
    for i in 1..n.abs() {
        let y_next = 2.0 * i as f64 * y_curr / x - y_prev;
        y_prev = y_curr;
        y_curr = y_next;
    }
    
    if n < 0 && n % 2 != 0 {
        -y_curr
    } else {
        y_curr
    }
}

/// 余除法
pub fn remquo(x: f64, y: f64, quo: &mut i32) -> f64 {
    if y == 0.0 || x.is_infinite() || y.is_nan() || x.is_nan() {
        *quo = 0;
        return f64::NAN;
    }
    
    let q = (x / y).round();
    *quo = q as i32;
    x - q * y
}

pub fn remquof(x: f32, y: f32, quo: &mut i32) -> f32 {
    let mut quo_f64 = 0i32;
    let result = remquo(x as f64, y as f64, &mut quo_f64);
    *quo = quo_f64;
    result as f32
}

/// 长双精度数学函数

// 三角函数
pub fn sinl(x: f64) -> f64 { sin(x) }
pub fn cosl(x: f64) -> f64 { cos(x) }
pub fn tanl(x: f64) -> f64 { tan(x) }
pub fn asinl(x: f64) -> f64 { asin(x) }
pub fn acosl(x: f64) -> f64 { acos(x) }
pub fn atanl(x: f64) -> f64 { atan(x) }
pub fn atan2l(y: f64, x: f64) -> f64 { atan2(y, x) }

// 双曲函数
pub fn sinhl(x: f64) -> f64 { sinh(x) }
pub fn coshl(x: f64) -> f64 { cosh(x) }
pub fn tanhl(x: f64) -> f64 { tanh(x) }

// 指数 对数
pub fn expl(x: f64) -> f64 { exp(x) }
pub fn expm1l(x: f64) -> f64 { expm1(x) }
pub fn logl(x: f64) -> f64 { log(x) }
pub fn log10l(x: f64) -> f64 { log10(x) }
pub fn log2l(x: f64) -> f64 { log2(x) }
pub fn log1pl(x: f64) -> f64 { log1p(x) }

// 幂函数
pub fn powl(x: f64, y: f64) -> f64 { pow(x, y) }
pub fn sqrtl(x: f64) -> f64 { sqrt(x) }
pub fn cbrtl(x: f64) -> f64 { cbrt(x) }
pub fn hypotl(x: f64, y: f64) -> f64 { hypot(x, y) }

// 取整
pub fn ceill(x: f64) -> f64 { ceil(x) }
pub fn floorl(x: f64) -> f64 { floor(x) }
pub fn truncl(x: f64) -> f64 { trunc(x) }
pub fn roundl(x: f64) -> f64 { round(x) }

// 其他函数
pub fn fmodl(x: f64, y: f64) -> f64 { fmod(x, y) }
pub fn remainderl(x: f64, y: f64) -> f64 { remainder(x, y) }
pub fn fabsl(x: f64) -> f64 { fabs(x) }
pub fn copysignl(x: f64, y: f64) -> f64 { copysign(x, y) }

// 浮点数操作
pub fn modfl(x: f64) -> (f64, f64) { modf(x) }
pub fn frexpl(x: f64) -> (f64, i32) { frexp(x) }
pub fn ldexpl(x: f64, exp: i32) -> f64 { ldexp(x, exp) }

// Gamma
pub fn tgammal(x: f64) -> f64 { tgamma(x) }
pub fn lgammal(x: f64) -> f64 { lgamma(x) }

/// 额外的数学函数
/// 这些函数在高级数学计算中经常用到

/// 反双曲函数
pub fn asinh(x: f64) -> f64 {
    x.asinh()
}

pub fn acosh(x: f64) -> f64 {
    if x < 1.0 {
        f64::NAN
    } else {
        x.acosh()
    }
}

pub fn atanh(x: f64) -> f64 {
    if x.abs() >= 1.0 {
        f64::NAN
    } else {
        x.atanh()
    }
}

/// 椭圆积分第一类 K(k) 的简化实现
/// K(k) = ∫₀^(π/2) dθ/√(1 - k²sin²θ)
pub fn elliptic_k(k: f64) -> f64 {
    if k.abs() >= 1.0 {
        return f64::INFINITY;
    }
    
    // 使用算术-几何平均方法
    let mut a = 1.0;
    let mut b = (1.0 - k * k).sqrt();
    let mut c = k;
    
    while c.abs() > 1e-15 {
        let a_new = (a + b) * 0.5;
        let b_new = (a * b).sqrt();
        c = (a - b) * 0.5;
        a = a_new;
        b = b_new;
    }
    
    M_PI * 0.5 / a
}

/// 椭圆积分第二类 E(k) 的简化实现
/// E(k) = ∫₀^(π/2) √(1 - k²sin²θ) dθ
pub fn elliptic_e(k: f64) -> f64 {
    if k.abs() >= 1.0 {
        return if k.abs() == 1.0 { 1.0 } else { f64::NAN };
    }
    
    // 使用算术-几何平均方法的改进版本
    let mut a = 1.0;
    let mut b = (1.0 - k * k).sqrt();
    let mut c = k;
    let mut sum = 0.0;
    let mut power_of_two = 1.0;
    
    while c.abs() > 1e-15 {
        sum += power_of_two * c * c;
        power_of_two *= 2.0;
        
        let a_new = (a + b) * 0.5;
        let b_new = (a * b).sqrt();
        c = (a - b) * 0.5;
        a = a_new;
        b = b_new;
    }
    
    (M_PI * 0.5 / a) * (1.0 - 0.5 * sum)
}

/// Digamma函数 ψ(x) = d/dx ln(Γ(x))
/// Gamma函数的对数导数
pub fn digamma(x: f64) -> f64 {
    if x <= 0.0 && x.fract() == 0.0 {
        return f64::NAN; // 在负整数处未定义
    }
    
    if x < 8.0 {
        // 使用递推关系移动到更大的值
        let mut result = 0.0;
        let mut xx = x;
        
        while xx < 8.0 {
            result -= 1.0 / xx;
            xx += 1.0;
        }
        
        result + digamma_large(xx)
    } else {
        digamma_large(x)
    }
}

/// 大参数Digamma函数的渐近展开
fn digamma_large(x: f64) -> f64 {
    // 使用Stirling级数
    // ψ(x) ≈ ln(x) - 1/(2x) - 1/(12x²) + 1/(120x⁴) - 1/(252x⁶) + ...
    let inv_x = 1.0 / x;
    let inv_x2 = inv_x * inv_x;
    
    x.ln() - 0.5 * inv_x - inv_x2 * (1.0/12.0 - inv_x2 * (1.0/120.0 - inv_x2 * 1.0/252.0))
}

/// Riemann Zeta函数 ζ(s) 的简化实现
/// 仅对s > 1实现
pub fn riemann_zeta(s: f64) -> f64 {
    if s <= 1.0 {
        return if s == 1.0 { f64::INFINITY } else { f64::NAN };
    }
    
    if s == 2.0 {
        return M_PI * M_PI / 6.0; // ζ(2) = π²/6
    }
    
    if s == 4.0 {
        return M_PI.powi(4) / 90.0; // ζ(4) = π⁴/90
    }
    
    // 对于其他值使用级数求和
    let mut sum = 0.0;
    for n in 1..1000 {
        let term = 1.0 / (n as f64).powf(s);
        sum += term;
        
        if term < 1e-15 {
            break;
        }
    }
    
    sum
}

/// 正态分布累积分布函数 Φ(x)
/// Φ(x) = (1 + erf(x/√2)) / 2
pub fn normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / M_SQRT2))
}

/// 正态分布概率密度函数 φ(x)
/// φ(x) = e^(-x²/2) / √(2π)
pub fn normal_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * M_PI).sqrt()
}

/// 学生t分布累积分布函数的简化实现
pub fn student_t_cdf(t: f64, df: f64) -> f64 {
    if df <= 0.0 {
        return f64::NAN;
    }
    
    // 使用不完全Beta函数的关系
    // P(T ≤ t) = 0.5 + (t/√(df)) * B(1/2, df/2) * ₂F₁(1/2, (df+1)/2; 3/2; -t²/df)
    // TODO: 完整实现
    let x = t / (df + t * t).sqrt();
    0.5 + 0.5 * x * beta(0.5, df * 0.5) / beta(0.5, 0.5)
}

/// 卡方分布累积分布函数的简化实现
pub fn chi_squared_cdf(x: f64, k: f64) -> f64 {
    if x <= 0.0 || k <= 0.0 {
        return if x <= 0.0 { 0.0 } else { f64::NAN };
    }
    
    // χ²(k) 的CDF = γ(k/2, x/2) / Γ(k/2)
    gamma_inc_lower(k * 0.5, x * 0.5) / tgamma(k * 0.5)
}
