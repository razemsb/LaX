<?php
$php = PHP_VERSION;
$os = PHP_OS_FAMILY;
$root = str_replace('\\', '/', dirname(__DIR__));
$sapi = PHP_SAPI;
$time = date('Y-m-d H:i:s');
$ext = get_loaded_extensions();
sort($ext);
?>
<!doctype html>
<html lang="ru">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>LaX</title>
  <style>
    :root { color-scheme: dark; }
    * { box-sizing: border-box; }
    body {
      margin: 0; min-height: 100vh;
      font-family: "Segoe UI", system-ui, sans-serif;
      background: radial-gradient(1200px 600px at 10% -10%, #163044 0%, transparent 50%),
                  radial-gradient(900px 500px at 100% 0%, #0f2a24 0%, transparent 45%),
                  #07090f;
      color: #e8eef7;
    }
    main { max-width: 880px; margin: 0 auto; padding: 72px 24px; }
    .kicker { color: #22d3ee; letter-spacing: .18em; font-size: 12px; font-weight: 700; }
    h1 { font-size: 56px; margin: 8px 0 12px; letter-spacing: -0.04em; }
    h1 span { color: #34d399; }
    p { color: #9aa8bd; line-height: 1.6; }
    .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-top: 32px; }
    .card {
      background: rgba(14,18,27,.8); border: 1px solid #1c2433;
      border-radius: 16px; padding: 16px 18px;
    }
    .card b { display: block; font-size: 12px; color: #7f8da3; font-weight: 600; margin-bottom: 6px; }
    a { color: #22d3ee; }
    code { font-family: ui-monospace, Consolas, monospace; font-size: 13px; }
  </style>
</head>
<body>
  <main>
    <div class="kicker">LOCAL STACK</div>
    <h1>LaX <span>is up</span></h1>
    <p>Портативная среда разработки. Документ-рут: <code><?= htmlspecialchars($root) ?>/www</code>. Проект <code>app</code> открывается как <code>http://localhost/app/</code>.</p>
    <div class="grid">
      <div class="card"><b>PHP</b><?= htmlspecialchars($php) ?><br><small><?= htmlspecialchars($sapi) ?></small></div>
      <div class="card"><b>OS</b><?= htmlspecialchars($os) ?></div>
      <div class="card"><b>Time</b><?= htmlspecialchars($time) ?></div>
      <div class="card"><b>Extensions</b><?= count($ext) ?></div>
    </div>
    <p style="margin-top:28px"><a href="/phpmyadmin">phpMyAdmin</a> · <a href="?info=1">phpinfo</a></p>
    <?php if (!empty($_GET['info'])): ?>
      <div class="card" style="margin-top:24px; overflow:auto"><?php phpinfo(); ?></div>
    <?php endif; ?>
  </main>
</body>
</html>
