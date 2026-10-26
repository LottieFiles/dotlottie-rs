---
default: minor
---

# feat: bezier path slots

Adds an experimental `BezierPath` slot type for overriding the geometry of path
(`ty: "sh"`) properties tagged with a slot id, backed by ThorVG's path slot support.

- `Player::set_bezier_path_slot` and `BezierPath` / `BezierPathSlot`, static or keyframed.
- `set_slots_str` and slots discovered from the animation recognise path values
  (`"bezier_path"`).
- Experimental `"type": "BezierPath"` theme rule with `vertices`, `inTangents`,
  `outTangents` and `closed`, pending a dotLottie theming spec proposal.
