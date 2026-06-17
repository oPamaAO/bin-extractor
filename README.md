# 🔓 Albion Online `.bin` Game Data Extractor

**bin-extract** — утилита для дешифровки и экспорта игровых данных Albion Online из бинарных `.bin` файлов (DES-CBC + gzip → XML → JSON).

---

## 🚀 Быстрый старт (Windows .exe)

Скачай `bin-extract.exe` со страницы [релиза](https://github.com/oPamaAO/bin-extractor/releases), открой **терминал** (PowerShell или cmd) в папке с exe и запусти:

```powershell
# Интерактивный режим — просто запусти
.\bin-extract.exe
```

Программа сама найдёт GameData, покажет меню:

```
╔══════════════════════════════════════╗
║   Albion Online .bin Data Extractor  ║
╚══════════════════════════════════════╝

  📂 GameData: C:\Program Files (x86)\AlbionOnline\...

  1. Extract all known .bin files    ← нажми 1 + Enter (извлечь всё)
  2. List available .bin files
  3. Extract a specific type
  4. Decrypt a .bin to XML
  5. Change GameData path
  q. Quit
  →
```

**Нажми `1` → Enter → Enter (папка `output`)** — и все данные извлечены в `output/`.

**Или через аргументы командной строки:**

```powershell
# Извлечь всё сразу
.\bin-extract.exe all "C:\Program Files (x86)\AlbionOnline\game\Albion-Online_Data\StreamingAssets\GameData" -o ./output

# Только предметы
.\bin-extract.exe items items.bin -o items.json

# Только строения
.\bin-extract.exe buildings buildings.bin -o buildings.json
```

---

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

---

## 📖 Подробное использование

### Команды

```powershell
# Интерактивное меню
.\bin-extract.exe

# Список .bin файлов в GameData
.\bin-extract.exe list "C:\Program Files (x86)\AlbionOnline\game\Albion-Online_Data\StreamingAssets\GameData"

# Извлечь всё в папку
.\bin-extract.exe all "путь/к/GameData" -o ./output

# Отдельные типы
.\bin-extract.exe items items.bin -o items.json
.\bin-extract.exe mobs mobs.bin -o mobs.json
.\bin-extract.exe spells spells.bin -o spells.json
.\bin-extract.exe buildings buildings.bin -o buildings.json
.\bin-extract.exe world cluster/world.bin -o world.json
.\bin-extract.exe gamedata gamedata.bin -o gamedata.json
.\bin-extract.exe localization localization.bin -o localization.json

# Только дешифровка (без парсинга XML)
.\bin-extract.exe decrypt encrypted.bin -o decrypted.xml
```

### Флаги

| Флаг | Описание |
|------|----------|
| `-g, --gamedata <path>` | Указать путь к GameData (или через `ALBION_GAMEDATA`) |
| `--format pretty\|compact` | Формат JSON (по умолч. pretty) |
| `-o, --output <file>` | Выходной файл |

### Переменные окружения

| Переменная | Описание |
|------------|----------|
| `ALBION_GAMEDATA` | Путь к GameData (автоопределение если не задан) |

### Путь к GameData

GameData лежит в папке с игрой. Типичные пути:

- Steam: `C:\Program Files (x86)\Steam\steamapps\common\Albion Online\game\Albion-Online_Data\StreamingAssets\GameData`
- Лаунчер: `C:\Program Files (x86)\AlbionOnline\game\Albion-Online_Data\StreamingAssets\GameData`

Программа умеет автоопределять оба варианта.

---

## 🛠 Установка из исходников (для разработчиков)

```bash
cargo install --git https://github.com/oPamaAO/bin-extractor
```

Или вручную:

```bash
git clone https://github.com/oPamaAO/bin-extractor.git
cd bin-extractor
cargo build --release
./target/release/bin-extract
```

---

## 🔧 Как это работает

```text
.bin файл
    ↓ DES-CBC(key=0x30EF724742F20432, iv=0x0EA6DC89DBEDDC4F)
GZip-сжатый XML
    ↓ gunzip
XML (UTF-8)
    ↓ quick-xml парсер
JSON
```

DES-ключ и IV восстановлены из клиента игры — ручной реверс-инжиниринг.

---

## 📄 Лицензия

MIT
