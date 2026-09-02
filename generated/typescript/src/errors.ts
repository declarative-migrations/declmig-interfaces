export type InterfaceErrorCode =
  | "schema_mismatch"
  | "invalid_format"
  | "empty_field"
  | "invalid_sha256"
  | "empty_phases"
  | "duplicate_phase_id"
  | "invalid_phase";

export class InterfaceError extends Error {
  readonly code: InterfaceErrorCode;

  constructor(code: InterfaceErrorCode, message: string) {
    super(message);
    this.code = code;
    this.name = "InterfaceError";
  }
}
