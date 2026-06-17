# 🔓 Albion Online `.bin` Game Data Extractor

**bin-extract** — утилита для дешифровки и экспорта игровых данных Albion Online из бинарных `.bin` файлов (DES-CBC + gzip → XML → JSON).

```bash
# Быстрый старт — интерактивный режим
bin-extract

# Извлечь всё одной командой
bin-extract all "путь/к/GameData" -o ./output

# Отдельные файлы
bin-extract items items.bin -o items.json
bin-extract world cluster/world.bin -o world.json
```

## 📦 Что извлекает

| Команда | Файл | Записей | Описание |
|---------|------|---------|----------|
| `items` | `items.bin` | 40 776 | Все предметы (оружие, броня, ресурсы, зелья...) |
| `mobs` | `mobs.bin` | 4 595 | Мобы и NPC |
| `spells` | `spells.bin` | 8 892 | Способности и заклинания |
| `buildings` | `buildings.bin` | 282 | Постройки и здания |
| `world` | `cluster/world.bin` | 2 613 | Кластеры мира (зоны, биомы) |
| `gamedata` | `gamedata.bin` | 76 | Настройки игры |
| `localization` | `localization.bin` | 562 516 | Переводы (EN/RU/DE/FR/...) |

**Всего: ~620 000 записей** из 7 файлов.

## 🚀 Установка

```bash
# Через cargo (из исходников)
cargo install --git https://github.com/oPamaAO/bin-extractor

# Или скачай готовый бинарник с релизов
# https://github.com/oPamaAO/bin-extractor/releases
```

## 📖 Использование

```bash
# Интерактивный режим (автоопределение GameData)
bin-extract

# Список всех .bin файлов
bin-extract list "C:\Program Files (x86)\AlbionOnline\game\Albion-Online_Data\StreamingAssets\GameData"

# Извлечь всё сразу
bin-extract all "путь/к/GameData" -o ./output

# Отдельные типы
bin-extract items items.bin -o items.json
bin-extract mobs mobs.bin -o mobs.json
bin-extract spells spells.bin -o spells.json
bin-extract buildings buildings.bin -o buildings.json
bin-extract world cluster/world.bin -o world.json
bin-extract gamedata gamedata.bin -o gamedata.json
bin-extract localization localization.bin -o localization.json
bin-extract decrypt encrypted.bin -o decrypted.xml    # только дешифровка
```

### Флаги

| Флаг | Описание |
|------|----------|
| `-g, --gamedata` | Путь к GameData (или `ALBION_GAMEDATA` env) |
| `--format pretty\|compact` | Формат JSON (по умолч. pretty) |
| `-o, --output` | Выходной файл |

### Переменные окружения

| Переменная | Описание |
|------------|----------|
| `ALBION_GAMEDATA` | Путь к GameData (автоопределение если не задан) |

## 🔧 Как это работает

1. **DES-CBC** — каждый `.bin` зашифрован 8-байтным ключом с инициализационным вектором (восстановлены из клиента игры)
2. **GZip** — после расшифровки данные сжаты gzip
3. **XML** — разархивированный XML парсится в структурированный JSON

```text
.bin файл
    ↓ DES-CBC(key, iv)
GZip-сжатый XML
    ↓ gunzip
XML (UTF-8)
    ↓ quick-xml парсер
JSON
```

## 🛠 Сборка из исходников

```bash
git clone https://github.com/oPamaAO/bin-extractor.git
cd bin-extractor
cargo build --release
./target/release/bin-extract
```

## 📄 Лицензия

MIT
