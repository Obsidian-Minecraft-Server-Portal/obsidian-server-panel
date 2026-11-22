export interface Health {
  status: string;
  version: string;
  setup_required: boolean;
  build: string;
  smtp_enabled: boolean;
  running_instances: number;
  sftp_port: number | null;
  updater_enabled: boolean;
}

export interface CurrentUser {
  id: number;
  username: string;
  email: string | null;
  is_admin: boolean;
  totp_enabled: boolean;
}

export interface WsServerMessage {
  topic: string;
  payload?: unknown;
  data?: unknown;
}

export interface WsClientMessage {
  op: "subscribe" | "unsubscribe";
  topic: string;
}

export interface Notification {
  id: number;
  kind: string;
  title: string;
  body: string | null;
  read: boolean;
  created_at: string;
}
