use serde_json::{json, Value};

const ATTO: &str = include_str!("../projections/atto/declmig_interfaces.atto");
const SCHEMA: &str = include_str!("../schema/v1/migrationplan.json");

#[test]
fn atto_plan_header_tracks_required_v1_json_schema_fields() {
    let schema: Value = serde_json::from_str(SCHEMA).unwrap();
    assert_eq!(schema["required"], json!(["id", "revision", "payload"]));
    for name in ["id", "revision"] {
        assert_eq!(schema["properties"][name]["type"], "string");
        assert_eq!(schema["properties"][name]["minLength"], 1);
        assert!(ATTO.contains(&format!("pub string {name}:")));
    }
    assert!(ATTO.contains("define class MigrationPlanHeader as"));
    assert!(ATTO.contains("migration_header_lengths_valid(int id_bytes, int revision_bytes)"));
    assert!(ATTO.contains("id_bytes <= 0"));
    assert!(ATTO.contains("revision_bytes > 0"));
    // The opaque JSON payload is not representable as a string or false typed model.
    assert!(!ATTO.contains("pub string payload:"));
}

#[test]
fn atto_health_projection_exposes_only_existing_fields() {
    assert!(ATTO.contains("define class HealthContract as"));
    for field in ["pub bool ok:", "pub string service:", "pub string protocol:"] {
        assert!(ATTO.contains(field));
    }
    assert!(ATTO.contains("supported_v1_protocol(int protocol_number)"));
}
