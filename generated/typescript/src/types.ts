export const PROTOCOL_VERSION = "1" as const;
export const SCHEMA_REVISION = "declmig-0001" as const;

export interface Health {
  ok: boolean;
  service: string;
  protocol: string;
}

export interface MigrationPlan {
  id: string;
  revision: string;
  payload: Record<string, unknown>;
}

