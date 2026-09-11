# 2026-09-02 peer-authority audit evidence

- `current-contract-parity.canonical.json` conforms to `schema/v1/peer-authority-parity-report.json` and is the machine-consumable gate result.
- `current-contract-parity.json` is the expanded human-review envelope. It records evaluation questions and release effects that are intentionally outside the minimal cross-language report contract.

Both files describe the same decision: `pause`. The expanded envelope must never be used as a substitute for the canonical report, and neither file may be changed to `proceed` without regenerating both independent authority lanes and attaching their exact artifact/tool digests.
