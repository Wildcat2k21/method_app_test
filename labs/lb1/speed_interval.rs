//! `core::rules::SpeedRules::interval` - покрытие ветвей.
//!
//! ```text
//! B1  level = score.checked_div(every)  → Some(level) / None (every == 0) → 0
//! B2  base.saturating_sub(step * level)  → разность / насыщение в 0
//! B3  .max(min)                          → разность / min
//! ```

use std::time::Duration;

use snake_game::core::SpeedRules;

fn ms(v: u64) -> Duration {
    Duration::from_millis(v)
}

fn rules(base: u64, min: u64, step: u64, every: u32) -> SpeedRules {
    SpeedRules { base: ms(base), min: ms(min), step: ms(step), every }
}

#[test]
fn b1_some_level_grows_every_n_points() {
    let r = rules(150, 60, 10, 5);
    assert_eq!(r.interval(4), ms(150));
    assert_eq!(r.interval(5), ms(140));
}

#[test]
fn b1_none_every_zero_never_speeds_up() {
    let r = rules(150, 60, 10, 0);
    assert_eq!(r.interval(1000), ms(150));
}

#[test]
fn b2_saturates_instead_of_underflow() {
    // step * level = 10 * 1000 мс > base: без saturating_sub была бы паника.
    let r = rules(150, 0, 10, 1);
    assert_eq!(r.interval(1000), Duration::ZERO);
}

#[test]
fn b3_clamped_to_min() {
    let r = rules(150, 60, 10, 5);
    assert_eq!(r.interval(45), ms(60)); // 150 − 90 = 60 - ровно минимум
    assert_eq!(r.interval(50), ms(60)); // 150 − 100 = 50 → 60
}

#[test]
fn b3_not_clamped_above_min() {
    let r = rules(150, 60, 10, 5);
    assert_eq!(r.interval(40), ms(70));
}
