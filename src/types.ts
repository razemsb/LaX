export interface LaxConfig {
  documentRoot: string;
  tld: string;
  autoVhost: boolean;
  webServer: string;
  apachePort: number;
  nginxPort: number;
  mysqlPort: number;
  phpVersion: string;
  mysqlVersion: string;
  nginxVersion: string;
  apacheVersion: string;
  phpCgiPorts: number[];
  autoStart: boolean;
  mysqlEnabled: boolean;
}

export interface ServiceInfo {
  id: string;
  name: string;
  running: boolean;
  pid: number | null;
  port: number | null;
  version: string;
  enabled: boolean;
}

export interface ProjectInfo {
  name: string;
  path: string;
  url: string;
  localhostUrl: string;
  hasPublic: boolean;
  kind: string;
  scripts: string[];
  hasPackage: boolean;
  hasComposer: boolean;
  hasNodeModules: boolean;
  hasVendor: boolean;
}

export interface PhpExtension {
  name: string;
  enabled: boolean;
  kind: string;
}

export interface Snapshot {
  root: string;
  config: LaxConfig;
  services: ServiceInfo[];
  projects: ProjectInfo[];
  phpVersions: string[];
  mysqlVersions: string[];
  nginxVersions: string[];
  apacheVersions: string[];
  hostsWritable: boolean;
  message: string | null;
}
