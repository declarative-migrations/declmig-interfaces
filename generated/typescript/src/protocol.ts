import { InterfaceError } from "./errors";
import type { MigrationPlan } from "./types";

export function parseMigrationPlan(
  id: string,
  revision: string,
  payload: Record<string, unknown>,
): MigrationPlan {
  if (!id.trim()) {
    throw new InterfaceError("empty_id");
  }
  if (!revision.trim()) {
    throw new InterfaceError("empty_revision");
  }
  return { id, revision, payload };
}

