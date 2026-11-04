import {
  Blocks,
  Database,
  Inbox,
  Layers,
  Server,
  Table2,
} from "lucide-vue-next";

export const SERVICE_NAMES: Record<string, string> = {
  apache: "Apache",
  nginx: "Nginx",
  mariadb: "MariaDB",
  php: "PHP",
  mailpit: "Mailpit",
  dbgate: "DbGate",
};

export const SERVICE_ICONS: Record<string, typeof Server> = {
  apache: Server,
  nginx: Layers,
  mariadb: Database,
  php: Blocks,
  mailpit: Inbox,
  dbgate: Table2,
};
