//! `core::highscore::parse` - покрытие ветвей.
//!
//! ```text
//! trim
//! B1  if text.is_empty() || !all_ascii_digits   → None
//!       C1 пусто            C2 есть не цифра
//! B2  text.parse::<u32>()                        → Ok → Some / Err → None
//!       (Err возможен только при переполнении: знаки и пустота отсечены в B1)
//! ```
//!
//! Почему нужна проверка C2, если есть `parse`: `"+5".parse::<u32>()` в Rust - это `Ok(5)`.
//! Тест `b1_c2_plus_sign_is_rejected` фиксирует это решение.

use snake_game::core::highscore::parse;

#[test]
fn b1_c1_empty() {
    assert_eq!(parse(""), None);
}

#[test]
fn b1_c1_only_whitespace() {
    assert_eq!(parse(" \n\t"), None);
}

#[test]
fn b1_c2_letters() {
    assert_eq!(parse("12a"), None);
}

#[test]
fn b1_c2_minus_sign() {
    assert_eq!(parse("-3"), None);
}

#[test]
fn b1_c2_plus_sign_is_rejected() {
    assert_eq!("+5".parse::<u32>(), Ok(5), "предпосылка теста: стандартный разбор принимает знак");
    assert_eq!(parse("+5"), None);
}

#[test]
fn b2_err_overflow() {
    assert_eq!(parse("4294967296"), None); // u32::MAX + 1
}

#[test]
fn b2_ok_max() {
    assert_eq!(parse("4294967295"), Some(u32::MAX));
}

#[test]
fn b2_ok_with_newline() {
    assert_eq!(parse("42\n"), Some(42));
}
