LoadModule fcgid_module "<<ROOT>>/etc/apache2/modules/mod_fcgid-2.3.9-win64-VC14.so"

<IfModule fcgid_module>
FcgidInitialEnv PATH "<<PHP_DIR>>;C:/Windows/system32;C:/Windows;C:/Windows/System32/Wbem;"
FcgidInitialEnv SystemRoot "C:/Windows"
FcgidInitialEnv SystemDrive "C:"
FcgidInitialEnv TEMP "<<ROOT>>/tmp"
FcgidInitialEnv TMP "<<ROOT>>/tmp"
FcgidInitialEnv windir "C:/Windows"
FcgidIOTimeout 36000
FcgidConnectTimeout 16
FcgidMaxRequestsPerProcess 0
FcgidMaxProcesses 50
FcgidMaxRequestLen 81310720
FcgidInitialEnv PHPRC "<<PHP_DIR>>"
FcgidInitialEnv PHP_FCGI_MAX_REQUESTS 0

<Files ~ "\.php$">
AddHandler fcgid-script .php
Options +ExecCGI
FcgidWrapper "<<PHP_DIR>>/php-cgi.exe" .php
</Files>
</IfModule>
