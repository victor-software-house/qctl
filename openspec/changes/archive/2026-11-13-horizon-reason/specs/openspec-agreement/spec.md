# Spec Delta

## REMOVED Requirements

### Requirement: A ledger that may carry keep_change_open declares schema version 5

**Reason**: Schema 6 renamed a horizon row's `kind` to `reason`, so version 5 is no longer current. The `horizon-rows` requirement "A ledger declares schema version 6" states the current version and how `qctl fmt` upgrades schema 4 and 5 to it.

**Migration**: Run `qctl fmt` once on a schema 4 or 5 ledger. An archived row keeps `keep_change_open` unchanged.
