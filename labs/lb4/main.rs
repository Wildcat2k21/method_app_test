//! ЛР4. Нефункциональное тестирование - производительность и стресс.
//!
//! Обычный прогон (`cargo test --test lb4`) выполняет только быстрые проверки бюджета.
//! Замеры с таблицами помечены `#[ignore]` и запускаются явно в release-сборке:
//!
//! ```bash
//! cargo test --release --test lb4 -- --ignored --nocapture --test-threads=1
//! ```
//!
//! `--test-threads=1` - чтобы замеры не делили ядра процессора между собой.

#[path = "../common/mod.rs"]
mod common;

mod performance;
