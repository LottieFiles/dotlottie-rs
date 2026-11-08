---
default: patch
---

# fix: state machines skip unknown entries and keep events pending

Unknown state, transition, guard, action, interaction and input entries are now skipped instead of failing the whole file. A fired event stays pending until a transition is taken or the state exits, so a transition that waits for a later input change can still see it.
