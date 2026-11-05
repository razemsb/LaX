import { ref } from "vue";

export type Locale = "ru" | "en" | "zh";

export const LOCALES: { id: Locale; native: string; label: string }[] = [
  { id: "ru", native: "Русский", label: "Русский" },
  { id: "en", native: "English", label: "English" },
  { id: "zh", native: "中文", label: "中文（简体）" },
];

const STRINGS = {
  "app.stack": { ru: "стек", en: "stack", zh: "服务栈" },
  "app.feedback": { ru: "фидбек / баг", en: "feedback / bug", zh: "反馈 / 缺陷" },
  "app.trayHint": { ru: "крестик прячет окно в трей", en: "the close button hides the window to the tray", zh: "点击关闭按钮会将窗口最小化到托盘" },

  "nav.overview": { ru: "Обзор", en: "Overview", zh: "概览" },
  "nav.projects": { ru: "Проекты", en: "Projects", zh: "项目" },
  "nav.php": { ru: "PHP", en: "PHP", zh: "PHP" },
  "nav.logs": { ru: "Логи", en: "Logs", zh: "日志" },
  "nav.settings": { ru: "Настройки", en: "Settings", zh: "设置" },

  "header.startAll": { ru: "Запустить все", en: "Start all", zh: "全部启动" },
  "header.stopAll": { ru: "Закрыть все", en: "Stop all", zh: "全部停止" },
  "header.start": { ru: "Старт", en: "Start", zh: "启动" },
  "header.stop": { ru: "Стоп", en: "Stop", zh: "停止" },

  "mode.mini": { ru: "Маленькое окно", en: "Small window", zh: "小窗口" },
  "mode.medium": { ru: "Среднее окно", en: "Medium window", zh: "中窗口" },
  "mode.large": { ru: "Большое окно", en: "Large window", zh: "大窗口" },
  "mode.expand": { ru: "Развернуть окно", en: "Expand window", zh: "展开窗口" },

  "banner.updateAvailable": { ru: "Доступна", en: "v{version} available", zh: "新版本 v{version}" },
  "banner.mb": { ru: "МБ", en: "MB", zh: "MB" },
  "banner.release": { ru: "релиз", en: "release", zh: "发布页" },
  "banner.update": { ru: "Обновить", en: "Update", zh: "更新" },
  "banner.updating": { ru: "обновляю…", en: "updating…", zh: "正在更新…" },
  "banner.portTaken": { ru: "Порт :{port} занят", en: "Port :{port} is taken", zh: "端口 :{port} 被占用" },
  "banner.switchTo8080": { ru: "Сменить на 8080", en: "Switch to 8080", zh: "切换到 8080" },

  "ctx.startAll": { ru: "Запустить все", en: "Start all", zh: "全部启动" },
  "ctx.stopAll": { ru: "Закрыть все", en: "Stop all", zh: "全部停止" },
  "ctx.mailpit": { ru: "Почта Mailpit", en: "Mailpit inbox", zh: "Mailpit 邮箱" },
  "ctx.wwwFolder": { ru: "Папка www", en: "www folder", zh: "www 文件夹" },
  "ctx.wwwTerminal": { ru: "Терминал www", en: "www terminal", zh: "www 终端" },

  "edit.cut": { ru: "Вырезать", en: "Cut", zh: "剪切" },
  "edit.copy": { ru: "Копировать", en: "Copy", zh: "复制" },
  "edit.paste": { ru: "Вставить", en: "Paste", zh: "粘贴" },
  "edit.selectAll": { ru: "Выделить всё", en: "Select all", zh: "全选" },

  "close.title": { ru: "Закрыть LaX?", en: "Close LaX?", zh: "关闭 LaX？" },
  "close.hint": { ru: "Стек можно оставить работать в трее.", en: "The stack can keep running in the tray.", zh: "服务栈可以继续保留在系统托盘中运行。" },
  "close.tray": { ru: "Свернуть в трей", en: "Minimize to tray", zh: "最小化到托盘" },
  "close.quit": { ru: "Закрыть полностью", en: "Quit completely", zh: "完全退出" },
  "close.cancel": { ru: "отмена", en: "cancel", zh: "取消" },

  "pill.online": { ru: "онлайн", en: "online", zh: "在线" },
  "pill.off": { ru: "выкл", en: "off", zh: "已停止" },
  "pill.running": { ru: "работает", en: "running", zh: "运行中" },
  "pill.stopped": { ru: "остановлен", en: "stopped", zh: "已停止" },

  "svc.service": { ru: "Сервис", en: "Service", zh: "服务" },
  "svc.version": { ru: "Версия", en: "Version", zh: "版本" },
  "svc.port": { ru: "Порт", en: "Port", zh: "端口" },
  "svc.pid": { ru: "PID", en: "PID", zh: "PID" },
  "svc.status": { ru: "Статус", en: "Status", zh: "状态" },
  "svc.intro": {
    ru: "Каждый процесс включается отдельно. Apache и Nginx делят порт 80 — одновременно работает только один веб-сервер.",
    en: "Each process is toggled separately. Apache and Nginx share port 80 — only one web server can run at a time.",
    zh: "每个进程均可单独开关。Apache 和 Nginx 共用 80 端口——同一时间只能运行一个 Web 服务器。",
  },

  "dash.sites": { ru: "Сайты", en: "Sites", zh: "网站" },
  "dash.sitesHint": { ru: "Проекты из", en: "Projects from", zh: "来自" },
  "dash.sitesHint2": { ru: "открываются как", en: "open as", zh: "的项目可通过以下地址访问" },
  "dash.mail": { ru: "почта", en: "mail", zh: "邮箱" },
  "dash.wwwFolder": { ru: "папка www", en: "www folder", zh: "www 文件夹" },
  "dash.mailpitMissing": {
    ru: "Mailpit не найден — письма mail() никуда не попадут. Запусти",
    en: "Mailpit not found — mail() letters will go nowhere. Run",
    zh: "未找到 Mailpit——mail() 发送的邮件将丢失。请运行",
  },
  "dash.mailpitOk": {
    ru: "PHP mail() ловит Mailpit. Laravel: MAIL_HOST=127.0.0.1 MAIL_PORT=1025 MAIL_MAILER=smtp",
    en: "PHP mail() is caught by Mailpit. Laravel: MAIL_HOST=127.0.0.1 MAIL_PORT=1025 MAIL_MAILER=smtp",
    zh: "PHP mail() 将由 Mailpit 接收。Laravel 配置：MAIL_HOST=127.0.0.1 MAIL_PORT=1025 MAIL_MAILER=smtp",
  },
  "dash.dbgateMissing": { ru: "DbGate не установлен —", en: "DbGate is not installed —", zh: "DbGate 未安装——" },
  "dash.dbgateDownload": { ru: "скачать (~350 МБ)", en: "download (~350 MB)", zh: "下载（约 350 MB）" },
  "dash.rootFolder": { ru: "Корень LaX", en: "LaX root", zh: "LaX 根目录" },
  "dash.terminal": { ru: "Терминал", en: "Terminal", zh: "终端" },
  "dash.browser": { ru: "Браузер", en: "Browser", zh: "浏览器" },
  "dash.mailpitBox": { ru: "Ящик Mailpit", en: "Mailpit inbox", zh: "Mailpit 邮箱" },
  "dash.databases": { ru: "Базы MariaDB", en: "MariaDB databases", zh: "MariaDB 数据库" },
  "dash.dbStartFirst": { ru: "Сначала запусти MariaDB.", en: "Start MariaDB first.", zh: "请先启动 MariaDB。" },
  "dash.dbStartBtn": { ru: "Запустить MariaDB", en: "Start MariaDB", zh: "启动 MariaDB" },
  "dash.dbName": { ru: "имя_базы", en: "db_name", zh: "数据库名" },
  "dash.dbCreate": { ru: "Создать", en: "Create", zh: "创建" },
  "dash.dbPick": { ru: "выбери базу", en: "pick a database", zh: "选择数据库" },
  "dash.dbImport": { ru: "импорт .sql", en: "import .sql", zh: "导入 .sql" },
  "dash.dbImporting": { ru: "импорт…", en: "importing…", zh: "正在导入…" },
  "dash.dbImported": { ru: "импортировано в {db}", en: "imported into {db}", zh: "已导入到 {db}" },
  "dash.projects": { ru: "Проекты", en: "Projects", zh: "项目" },
  "dash.all": { ru: "все", en: "all", zh: "全部" },
  "dash.empty": { ru: "Пока пусто.", en: "Nothing here yet.", zh: "暂无内容。" },

  "proj.name": { ru: "имя-проекта", en: "project-name", zh: "项目名称" },
  "proj.create": { ru: "Создать", en: "Create", zh: "创建" },
  "proj.creating": { ru: "создаю…", en: "creating…", zh: "正在创建…" },
  "proj.downloading": { ru: "качаю…", en: "downloading…", zh: "正在下载…" },
  "proj.hintLaravel": { ru: "в терминале: composer create-project laravel/laravel", en: "in terminal: composer create-project laravel/laravel", zh: "在终端中执行：composer create-project laravel/laravel" },
  "proj.hintVite": { ru: "в терминале: npm create vite — шаблон Vue", en: "in terminal: npm create vite — Vue template", zh: "在终端中执行：npm create vite——Vue 模板" },
  "proj.hintWp": { ru: "скачает latest.zip с wordpress.org в www", en: "downloads latest.zip from wordpress.org into www", zh: "将从 wordpress.org 下载 latest.zip 到 www" },
  "proj.hintPhp": { ru: "пустой index.php в www/имя · ПКМ по карточке — npm / composer", en: "empty index.php in www/name · right-click the card for npm / composer", zh: "在 www/名称 中创建空 index.php · 右键卡片使用 npm / composer" },
  "proj.search": { ru: "поиск проекта, kind, скрипт…  Ctrl+F", en: "search project, kind, script…  Ctrl+F", zh: "搜索项目、类型、脚本…  Ctrl+F" },
  "proj.wwwEmpty": { ru: "Папка www пустая. Проект откроется как http://localhost/имя/", en: "The www folder is empty. A project will open as http://localhost/name/", zh: "www 文件夹为空。创建的项目将通过 http://localhost/名称/ 访问" },
  "proj.emptyAction": { ru: "Создать первый проект", en: "Create your first project", zh: "创建第一个项目" },
  "proj.notFound": { ru: "Ничего не найдено.", en: "Nothing found.", zh: "没有找到匹配项。" },
  "proj.nodeMissing": {
    ru: "Свой Node не найден в bin/node — npm-кнопки возьмут системный, если он есть. Положи туда node.exe или запусти",
    en: "No bundled Node in bin/node — npm buttons will fall back to a system Node if present. Put node.exe there or run",
    zh: "未在 bin/node 中找到内置 Node——npm 按钮将回退到系统 Node（如果有）。请将 node.exe 放入该目录，或运行",
  },
  "proj.site": { ru: "сайт", en: "site", zh: "网站" },
  "proj.folder": { ru: "папка", en: "folder", zh: "文件夹" },
  "proj.term": { ru: "терминал", en: "terminal", zh: "终端" },
  "proj.ctxCommands": { ru: "ПКМ — {n} команд (npm / composer)", en: "Right-click — {n} commands (npm / composer)", zh: "右键——{n} 个命令（npm / composer）" },
  "proj.openSite": { ru: "Открыть сайт", en: "Open site", zh: "打开网站" },
  "proj.ctxFolder": { ru: "Папка", en: "Folder", zh: "文件夹" },
  "proj.ctxTerminal": { ru: "Терминал", en: "Terminal", zh: "终端" },
  "share.start": { ru: "Поделиться сайтом", en: "Share site", zh: "分享网站" },
  "share.stop": { ru: "Закрыть публичный доступ", en: "Stop public access", zh: "关闭公开访问" },
  "share.warning": {
    ru: "Сайт станет доступен любому, у кого будет ссылка. Продолжить?",
    en: "Anyone with the link will be able to open the site. Continue?",
    zh: "任何获得链接的人都将可以访问该网站。是否继续？",
  },
  "share.dialogTitle": { ru: "Открыть публичный доступ?", en: "Open public access?", zh: "开放公开访问？" },
  "share.dialogHint": {
    ru: "Сайт получит адрес trycloudflare.com — его сможет открыть любой, у кого будет ссылка. Доступ можно закрыть в любой момент.",
    en: "The site gets a trycloudflare.com address — anyone with the link can open it. You can close access at any time.",
    zh: "网站将获得 trycloudflare.com 地址——任何获得链接的人都可以访问。可以随时关闭访问。",
  },
  "share.confirm": { ru: "Открыть доступ", en: "Open access", zh: "开放访问" },
  "proj.phpDefault": { ru: "PHP по умолчанию", en: "Default PHP", zh: "默认 PHP" },
  "share.public": { ru: "публично", en: "public", zh: "公开" },
  "share.opening": { ru: "открываю доступ…", en: "opening access…", zh: "正在开启访问…" },
  "share.bannerTitle": { ru: "Сайт открыт извне", en: "Site is publicly accessible", zh: "网站已对外开放" },
  "share.open": { ru: "открыть", en: "open", zh: "打开" },
  "share.close": { ru: "закрыть", en: "close", zh: "关闭" },
  "set.autoVhost": { ru: "Автодомены", en: "Auto domains", zh: "自动域名" },
  "set.autoVhostHint": { ru: "папка в www → сайт по адресу имя.домен", en: "a folder in www → site at name.domain", zh: "www 中的文件夹 → 通过 名称.域名 访问" },
  "set.autoVhostHosts": { ru: "hosts недоступен для записи — домены работать не будут. Запусти LaX от имени администратора.", en: "hosts file is not writable — domains will not resolve. Run LaX as administrator.", zh: "hosts 文件不可写——域名将无法解析。请以管理员身份运行 LaX。" },
  "set.tld": { ru: "Домен", en: "Domain", zh: "域名" },

  "php.version": { ru: "Версия", en: "Version", zh: "版本" },
  "php.quick": { ru: "Быстрый php.ini", en: "Quick php.ini", zh: "快速 php.ini" },
  "php.quickHint": {
    ru: "пишется сразу; если стек онлайн — PHP перезапустится",
    en: "written immediately; if the stack is online, PHP restarts",
    zh: "立即写入；若服务栈正在运行，PHP 将自动重启",
  },
  "php.errorsInBrowser": { ru: "ошибки в браузере", en: "errors in the browser", zh: "在浏览器中显示错误" },
  "php.xdebugIde": { ru: "IDE слушай :{port} · CGI :9003", en: "IDE listens :{port} · CGI :9003", zh: "IDE 监听 :{port} · CGI :9003" },
  "php.xdebugMissing": { ru: "dll не найдена в ext/", en: "dll not found in ext/", zh: "ext/ 中未找到 dll" },
  "php.xdebugMissingTitle": { ru: "нет php_xdebug в ext/", en: "no php_xdebug in ext/", zh: "ext/ 中没有 php_xdebug" },
  "php.memory": { ru: "memory_limit", en: "memory_limit", zh: "memory_limit" },
  "php.upload": { ru: "upload_max_filesize", en: "upload_max_filesize", zh: "upload_max_filesize" },
  "php.post": { ru: "post", en: "post", zh: "post" },
  "php.extensions": { ru: "Расширения", en: "Extensions", zh: "扩展" },
  "php.extCount": {
    ru: "{n} включено · после списка расширений лучше перезапустить стек",
    en: "{n} enabled · restart the stack after changing extensions",
    zh: "已启用 {n} 个 · 修改扩展后建议重启服务栈",
  },
  "php.search": { ru: "поиск расширения  Ctrl+F", en: "search extension  Ctrl+F", zh: "搜索扩展  Ctrl+F" },
  "php.ini": { ru: "php.ini", en: "php.ini", zh: "php.ini" },
  "php.nothingFound": { ru: "ничего не найдено", en: "nothing found", zh: "没有找到匹配项" },

  "logs.search": { ru: "поиск в логе  Ctrl+F", en: "search in log  Ctrl+F", zh: "在日志中搜索  Ctrl+F" },
  "logs.refresh": { ru: "Обновить", en: "Refresh", zh: "刷新" },
  "banner.close": { ru: "закрыть", en: "close", zh: "关闭" },
  "store.installDbgate": { ru: "ставлю DbGate… ~350 МБ, несколько минут", en: "installing DbGate… ~350 MB, a few minutes", zh: "正在安装 DbGate……约 350 MB，需要几分钟" },
  "store.downloadingUpdate": { ru: "скачиваю обновление…", en: "downloading update…", zh: "正在下载更新…" },

  "set.language": { ru: "Язык", en: "Language", zh: "语言" },
  "set.languageHint": { ru: "интерфейс — сразу, без перезапуска", en: "interface — instantly, no restart", zh: "界面语言——即时生效，无需重启" },
  "set.window": { ru: "Окно", en: "Window", zh: "窗口" },
  "set.windowHint": { ru: "размер окна — сразу", en: "window size — applies instantly", zh: "窗口大小——即时生效" },
  "set.theme": { ru: "Тема", en: "Theme", zh: "主题" },
  "set.themeHint": { ru: "меняется сразу, стек не трогает", en: "applies instantly, does not touch the stack", zh: "立即生效，不影响服务栈" },
  "set.experimental": { ru: "эксп.", en: "exp.", zh: "试验" },
  "set.docRoot": { ru: "Корень сайтов", en: "Sites root", zh: "网站根目录" },
  "set.webServer": { ru: "Веб-сервер", en: "Web server", zh: "Web 服务器" },
  "set.dbPanel": { ru: "Панель баз", en: "Database panel", zh: "数据库面板" },
  "set.pmaHint": { ru: "в zip, через Apache / Nginx", en: "in the zip, via Apache / Nginx", zh: "包含在压缩包中，经由 Apache / Nginx 访问" },
  "set.dbgateHint": { ru: "веб, порт 8030", en: "web, port 8030", zh: "Web 界面，端口 8030" },
  "set.dbgateMissing": { ru: "не в zip · скачать ~350 МБ", en: "not in the zip · download ~350 MB", zh: "不在压缩包中 · 需下载约 350 MB" },
  "set.dbNote": {
    ru: "phpMyAdmin едет в релизе. DbGate ставится кнопкой выше, один раз, после установки или обновления.",
    en: "phpMyAdmin ships with the release. DbGate is installed with the button above, once, after installing or updating.",
    zh: "phpMyAdmin 随版本发布。DbGate 可通过上方按钮安装，只需一次，在安装或更新后进行。",
  },
  "set.downloadDbgate": { ru: "Скачать DbGate", en: "Download DbGate", zh: "下载 DbGate" },
  "set.startAllBtn": { ru: "Кнопка «Запустить все»", en: "“Start all” button", zh: "“全部启动”按钮" },
  "set.startAllHint": {
    ru: "только отмеченные службы. phpMyAdmin идёт через веб-сервер, отдельный процесс не нужен.",
    en: "only checked services. phpMyAdmin goes through the web server, no separate process needed.",
    zh: "仅启动勾选的服务。phpMyAdmin 经由 Web 服务器访问，无需单独进程。",
  },
  "set.webPlusPhp": { ru: "Веб-сервер ({x} + PHP)", en: "Web server ({x} + PHP)", zh: "Web 服务器（{x} + PHP）" },
  "set.phpmyadminButDbgate": {
    ru: "Панель баз — phpMyAdmin, но DbGate всё равно поднимется. Сними галку, если не нужен.",
    en: "Database panel is phpMyAdmin, but DbGate will still start. Uncheck it if not needed.",
    zh: "数据库面板为 phpMyAdmin，但 DbGate 仍会启动。如不需要请取消勾选。",
  },
  "set.autoStart": { ru: "При открытии LaX сразу жать «Запустить все»", en: "Press “Start all” automatically when LaX opens", zh: "打开 LaX 时自动执行“全部启动”" },
  "set.save": { ru: "Сохранить", en: "Save", zh: "保存" },
  "set.configs": { ru: "Конфиги", en: "Configs", zh: "配置文件" },
  "set.about": { ru: "О программе", en: "About", zh: "关于" },
  "set.tagline": { ru: "портативный локальный стек", en: "portable local stack", zh: "便携式本地开发环境" },
  "set.checkUpdates": { ru: "Проверить обновления", en: "Check for updates", zh: "检查更新" },
  "set.updateFound": {
    ru: "Есть v{version}. Можно поставить сверху, www и базы не затрутся.",
    en: "v{version} is available. It installs on top — www and databases are kept.",
    zh: "发现新版本 v{version}。可直接覆盖安装，www 和数据库不会被清除。",
  },
  "set.repo": { ru: "репозиторий", en: "repository", zh: "仓库" },
  "set.issues": { ru: "задачи", en: "issues", zh: "问题跟踪" },

  "theme.noir": { ru: "матовый чёрный", en: "matte black", zh: "哑光黑" },
  "theme.paper": { ru: "тёплая бумага", en: "warm paper", zh: "暖调纸张" },
  "theme.midnight": { ru: "ночная синь", en: "night blue", zh: "夜幕蓝" },
  "theme.quartz": { ru: "холодный белый", en: "cold white", zh: "冷调白" },
  "theme.glass": { ru: "ледяной синий", en: "icy blue", zh: "冰蓝" },
  "theme.obsidian": { ru: "тёмная лава", en: "dark lava", zh: "暗岩" },
} as const;

export type MsgKey = keyof typeof STRINGS;

export function normalizeLocale(id?: string | null): Locale {
  const v = (id ?? "").trim().toLowerCase();
  if (v === "en" || v === "eng" || v === "english") return "en";
  if (v === "zh" || v === "cn" || v === "zh-cn" || v === "chinese") return "zh";
  return "ru";
}

function detectLocale(): Locale {
  try {
    return normalizeLocale(localStorage.getItem("lax-locale"));
  } catch {
    return "ru";
  }
}

export const locale = ref<Locale>(detectLocale());

export function setLocale(next: Locale) {
  if (locale.value === next) return;
  locale.value = next;
  document.documentElement.lang = next === "zh" ? "zh-CN" : next;
  try {
    localStorage.setItem("lax-locale", next);
  } catch {
    /* ignore */
  }
}

export function bootLocale() {
  try {
    setLocale(detectLocale());
  } catch {
    /* ignore */
  }
}

export function t(key: MsgKey, params?: Record<string, string | number>): string {
  const row = STRINGS[key];
  let s: string = row?.[locale.value] ?? row?.ru ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      s = s.replaceAll(`{${k}}`, String(v));
    }
  }
  return s;
}
