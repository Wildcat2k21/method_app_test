//! Рекорд в текстовом файле: одно неотрицательное число.

use std::fs;
use std::io;
use std::path::Path;

/// Рекорд из содержимого файла. Пробелы и перевод строки по краям допустимы,
/// всё остальное (пустая строка, знак, дробь, переполнение `u32`) - `None`.
pub fn parse(text: &str) -> Option<u32> {
    let text = text.trim();
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// Рекорд из файла; отсутствующий или повреждённый файл даёт 0.
pub fn load(path: &Path) -> u32 {
    fs::read_to_string(path).ok().as_deref().and_then(parse).unwrap_or(0)
}

pub fn save(path: &Path, best: u32) -> io::Result<()> {
    fs::write(path, format!("{best}\n"))
}
