#!/usr/bin/env bash
# Рендер диаграмм PlantUML: каждый docs/uml/*.puml → .png и .svg рядом.
# Нужен локальный PlantUML-сервер (см. README.md, раздел «Инструменты отчётов»).
#
#   scripts/uml.sh                        все диаграммы
#   scripts/uml.sh path/to/diagram.puml   одна диаграмма
#   PLANTUML_SERVER=http://host:port scripts/uml.sh

set -euo pipefail
cd "$(dirname "$0")/.."

server=${PLANTUML_SERVER:-http://localhost:8080}
curl -sf -o /dev/null "$server/" || { echo "PlantUML-сервер $server недоступен" >&2; exit 1; }

files=("$@")
[[ ${#files[@]} -gt 0 ]] || files=(docs/uml/*.puml)

for src in "${files[@]}"; do
    for fmt in png svg; do
        dst="${src%.puml}.$fmt"
        curl -sf -X POST -H "Content-Type: text/plain; charset=utf-8" \
            --data-binary "@$src" "$server/$fmt" -o "$dst"
        echo "$dst ($(du -h "$dst" | cut -f1))"
    done
done
