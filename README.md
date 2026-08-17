<div align="center">

<img src="logo.svg" width="128" height="128" alt="LaX">

# LaX

**Портативный локальный стек.**  
Распаковал → `lax.exe` → `http://localhost/папка/`

Apache · Nginx · MariaDB · PHP · Mailpit · GUI

[![release](https://img.shields.io/github/v/release/razemsb/LaX?color=E02430&label=release&style=for-the-badge)](https://github.com/razemsb/LaX/releases)
[![windows](https://img.shields.io/badge/Windows-стек-111?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/razemsb/LaX/releases)
[![license](https://img.shields.io/badge/portable-без%20установщика-E02430?style=for-the-badge)](https://github.com/razemsb/LaX/releases/latest)

[Скачать zip](https://github.com/razemsb/LaX/releases/latest) · [Issues](https://github.com/razemsb/LaX/issues) · [Фидбек](https://github.com/razemsb/LaX/issues/new)

</div>

---

<p align="center">
  <a href="#-установка">установка</a> ·
  <a href="#-пять-минут">старт</a> ·
  <a href="#-экраны">экраны</a> ·
  <a href="#-проекты">проекты</a> ·
  <a href="#-php">php</a> ·
  <a href="#-почта-и-базы">почта</a> ·
  <a href="#-сборка">сборка</a>
</p>

| | | |
| :---: | :---: | :---: |
| **zip → exe** | **без Docker** | **www = сайты** |
| никакого установщика и служб | стек едет с собой в папке | `localhost/имя/` сразу |

> **Windows** — полный стек.  
> **Linux** — пока только оболочка (AppImage), бинарников Apache/PHP там нет.

---

## Установка

```
релизы  →  LaX-x.y.z.zip  →  C:\LaX\  →  lax.exe
```

1. Скачай архив с [релизов](https://github.com/razemsb/LaX/releases/latest).
2. Распакуй куда угодно, **не** в `Program Files` — туда часто нельзя писать.
3. Запусти `lax.exe`. Брандмауэр может спросить сеть: Apache слушает `:80`.

Крестик прячет окно в **трей**. Полный выход — *Quit* у иконки.

Обновление уже стоящего LaX: баннер сверху или **Настройки → Проверить обновления**.  
`www`, базы и логи не затираются.

---

## Пять минут

```
Запустить все  →  localhost  →  Проекты  →  создать  →  открыть сайт
```

| шаг | действие |
| ---: | --- |
| 1 | **Запустить все** — или клик по карточке Apache / MariaDB |
| 2 | **localhost** — корень `www` (FileManager) |
| 3 | **Проекты** → имя → шаблон → **Создать** |
| 4 | `http://localhost/имя/` · Laravel: `…/имя/public/` |

`:80` занят (IIS, Skype, другой Apache)? Баннер предложит **8080** — адреса станут `http://localhost:8080/имя/`.

ПКМ по карточке проекта — `npm run`, composer, папка, VS Code.  
**Ctrl+F** — поиск на экране (проекты, расширения, логи).

---

## Экраны

```
Обзор     сервисы · сайты · базы · превью
Проекты   шаблоны · ПКМ-команды · поиск
PHP       версия · php.ini · Xdebug
Логи      Apache / Nginx / MariaDB / PHP / Mailpit
Настройки порты · автостарт · обновления
```

Клик по карточке сервиса — старт / стоп. Зелёная обводка = онлайн.

---

## Проекты

Всё живёт в `www`. Служебные папки (`api`, `vendor`, `node_modules`…) в списке скрыты.

| | шаблон | что получишь |
| :---: | --- | --- |
| ◆ | **PHP** | сразу `index.php` |
| ◈ | **Laravel** | терминал: `composer create-project` |
| ◇ | **Vite** | `npm create vite` (Vue) · нужен Node в `bin/node` |
| ○ | **WordPress** | качает `latest.zip` с wordpress.org |

Пока composer/npm крутятся, список подхватит папку сам.

На карточке: сайт · папка · терминал · VS Code.  
Остальное — **правая кнопка**: `npm install`, `composer install`, все `npm run …`.

Vite в dev — кнопка `:5173`. Сборка как подпапка Apache — это уже `base` в `vite.config`.

---

## PHP

Быстрый `php.ini` на экране **PHP** — пишется сразу. Если стек онлайн, PHP перезапускается.

```
display_errors     вкл / выкл
memory_limit       128M   256M   512M   1G
upload_max         8M     32M    64M    128M     (+ post_max_size)
Xdebug             IDE :9000     CGI :9003 не трогаем
```

В PhpStorm / VS Code слушай **9000**, не 9003. Нет `php_xdebug` в `ext/` — тумблер серый.

Версии лежат в `bin/php/*`. Полный файл — кнопка `php.ini`.

---

## Почта и базы

<div align="center">

| Mailpit | |
| :---: | --- |
| ящик | [http://localhost:8025](http://localhost:8025) |
| SMTP | `127.0.0.1:1025` |

</div>

```env
MAIL_MAILER=smtp
MAIL_HOST=127.0.0.1
MAIL_PORT=1025
MAIL_ENCRYPTION=null
```

Нет `bin/mailpit/mailpit.exe` — `mail()` уходит в никуда. В репозитории: `npm run fetch-tools`.

MariaDB из Обзора: создать базу · импорт `.sql` · phpMyAdmin на выбранную.  
Дампа из GUI пока нет.

phpMyAdmin → [http://localhost/phpmyadmin/](http://localhost/phpmyadmin/)

---

## Конфиг

То же самое в GUI. Файл: `usr/lax.toml`

```toml
documentRoot  = "www"
webServer     = "apache"       # apache | nginx
apachePort    = 80
nginxPort     = 80
mysqlPort     = 3306
phpVersion    = "php-trash-8.2"
autoStart     = false          # поднять стек при открытии
mysqlEnabled  = true
```

`tld` и `autoVhost` уже в файле — красивые `проект.test` ещё не включены.

<details>
<summary><strong>Порты</strong></summary>

| | порт |
| --- | :---: |
| Apache / Nginx | `80` |
| MariaDB | `3306` |
| PHP-CGI (Nginx) | `9003` `9004` |
| Mailpit UI | `8025` |
| Mailpit SMTP | `1025` |
| Xdebug → IDE | `9000` |
| Vite dev | `5173` |

</details>

<details>
<summary><strong>Папки внутри LaX</strong></summary>

```
LaX/
├── lax.exe
├── www/              сайты, FileManager в корне
├── bin/              Apache · Nginx · MariaDB · PHP · Node · Mailpit · Composer
├── data/mariadb/     базы
├── etc/              apache2 · nginx · phpMyAdmin
├── usr/lax.toml      настройки
├── logs/             стек и падения GUI
└── tmp/              обновления, wordpress.zip
```

В терминал подмешивается PATH: текущий PHP и портативный Node.

</details>

---

## Сборка

Нужны Windows 10+, **Node 22**, **Rust stable**, VS 2022 Build Tools (C++).

`bin/` в git не лежит — положи рядом релиз или прогони `scripts/bootstrap.ps1` из Laragon.

```powershell
git clone https://github.com/razemsb/LaX.git
cd LaX
npm install
npm run fetch-tools     # Mailpit + Node → bin/
npm run lax             # окно + hot reload
npm run build:exe       # lax.exe в корне
npm run pack            # портативное дерево → pack/
```

`scripts/zip-release.ps1` собирает `LaX-<версия>.zip` из `pack/`.

| команда | |
| --- | --- |
| `npm run lax` | GUI + hot reload |
| `npm run dev` | только Vite, без окна |
| `npm run build` | типы + `dist/` |
| `npm run build:exe` | релизный exe |
| `npm run pack` | полный каталог |
| `npm run fetch-tools` | Mailpit и Node |

Linux AppImage: тег `v*` или Actions → **Linux**. Это оболочка, не стек.

---

## Если молчит

| симптом | что проверить |
| --- | --- |
| порт 80 занят | баннер или порт в Настройках |
| сайт 404 | стек онлайн? папка в `www`? Laravel → `/public/` |
| `mail()` пусто | Mailpit выключен или нет бинарника |
| нет npm / composer | нет `package.json` / `composer.json`, либо нет Node |
| Xdebug молчит | IDE на `:9000`, не на `:9003` |
| обновление зависло (≤ 0.1.3) | снять `lax.exe`, поставить [0.1.4+](https://github.com/razemsb/LaX/releases) |

---

<div align="center">

баг или идея → [issues](https://github.com/razemsb/LaX/issues/new)  
в приложении это **фидбек / баг**

<img src="logo.svg" width="36" height="36" alt="">

</div>
