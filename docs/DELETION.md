# Deletion and forgetting

v0.1 intentionally does not expose permanent purge in the desktop or CLI. A correct permanent purge must cover the current SQLCipher materialized view, FTS rows, encrypted Source object key, checkpoints, reachable backup manifests, external replicas, and restore verification. A one-row delete cannot honestly prove cryptographic erasure from older backups.

Until the purge state machine is implemented, owners can revoke a Memory from normal use and can destroy an entire Vault only by deliberately deleting all Vault copies, every Backup, and the Recovery Kit outside the application. Pensive does not attempt a broad recursive delete.

Remote deletion cannot recover Context Packs already exported, plaintext copied by the owner, or data obtained by a compromised/lost unlocked device.
