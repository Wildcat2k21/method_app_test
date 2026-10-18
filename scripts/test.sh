#!/usr/bin/env bash
# Запуск тестов лабораторных работ.
#
#   scripts/test.sh                  все тесты
#   scripts/test.sh lb3              тесты одной работы
#   scripts/test.sh lb3 --report     то же + протокол в labs/lb3/results.txt
#   scripts/test.sh lb4 --perf       замеры ЛР4 (release, --ignored, по одному потоку)
#   scripts/test.sh lb4 --perf --report
#
# ЛР5 - модульные тесты внутри библиотеки (cargo test --lib lb5),
# остальные работы - отдельные тестовые цели (cargo test --test lbN).

set -euo pipefail
cd "$(dirname "$0")/.."

lab=${1:-all}
shift || true
report=false
perf=false
for arg in "$@"; do
    case $arg in
        --report) report=true ;;
        --perf) perf=true ;;
        *) echo "неизвестный флаг: $arg" >&2; exit 1 ;;
    esac
done

case $lab in
    all) cmd=(cargo test) ;;
    lb5) cmd=(cargo test --lib lb5) ;;
    lb[1-6]) cmd=(cargo test --test "$lab") ;;
    *) echo "использование: $0 [all|lb1..lb6] [--report] [--perf]" >&2; exit 1 ;;
esac

if $perf; then
    [[ $lab == lb4 ]] || { echo "--perf есть только у lb4" >&2; exit 1; }
    cmd=(cargo test --release --test lb4 -- --ignored --nocapture --test-threads=1)
fi

if ! $report; then
    exec "${cmd[@]}"
fi

[[ $lab != all ]] || { echo "--report пишет протокол одной работы: укажите lbN" >&2; exit 1; }
out="labs/$lab/results$($perf && echo _perf || true).txt"
{
    echo "Протокол: $lab"
    echo "Дата:     $(date '+%Y-%m-%d %H:%M:%S')"
    echo "Система:  $(uname -srm)"
    echo "Rust:     $(rustc --version)"
    echo "Команда:  ${cmd[*]}"
    echo
    CARGO_TERM_COLOR=never "${cmd[@]}" 2>&1 | grep -vE '^\s*(Compiling|Blocking|Finished|Running) '
} | tee "$out"
echo "протокол сохранён: $out"
