/// Easing functions used to interpolate zoom level and viewport position.
///
/// All functions map `t ∈ [0.0, 1.0]` → `[0.0, 1.0]`.

/// Ease in-out cubic — smooth start and end, fast in the middle.
///
/// Good default for zoom transitions.
#[inline]
pub fn ease_in_out_cubic(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0_f64).powi(3) / 2.0
    }
}

/// Ease out expo — snappy start, gentle landing.
#[inline]
pub fn ease_out_expo(t: f64) -> f64 {
    if t >= 1.0 {
        1.0
    } else {
        1.0 - 2.0_f64.powf(-10.0 * t)
    }
}

/// Linear — no easing (useful for testing).
#[inline]
pub fn linear(t: f64) -> f64 {
    t.clamp(0.0, 1.0)
}

/// Linearly interpolate between `a` and `b` using normalised `t`.
#[inline]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
