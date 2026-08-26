import 'errors.dart';
import 'models.dart';

const protocolVersion = '1';
const schemaRevision = 'declmig-0001';

MigrationPlan parseMigrationPlan(String id, String revision, Map<String, Object?> payload) {
  if (id.trim().isEmpty) {
    throw const InterfaceException('empty_id');
  }
  if (revision.trim().isEmpty) {
    throw const InterfaceException('empty_revision');
  }
  return MigrationPlan(id: id, revision: revision, payload: payload);
}

