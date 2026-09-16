---
schema_version: 1
artifact: delivery-map
design_ref: docs/supreme/specs/2026-09-16-transcript-projection-settlement/design.md
outcomes:
  framework-projection-authority:
    ships: Framework provides atomic idempotent transcript projection, durable
      pending settlement, and closed compact/finalize/recovery/clear lifecycle
      behavior.
    depends_on: []
  sdk-projection-bridge:
    ships: Independent SDK exposes ConversationStore epoch/projection/delete and
      RuntimeStateStore CAS/retirement contracts through protocol, Host, and all
      three language clients against the merged framework revision.
    depends_on:
      - framework-projection-authority
  cross-repository-closure:
    ships: "Framework and SDK semantic evidence, superproject pointers, and Issue
      #106 reflect the fully merged transcript projection settlement."
    depends_on:
      - framework-projection-authority
      - sdk-projection-bridge
design_revision: sha256:a9f761bbab67efdd47e47433e2469eebf65238e350d1aef7db5ed33408885fa9
---
