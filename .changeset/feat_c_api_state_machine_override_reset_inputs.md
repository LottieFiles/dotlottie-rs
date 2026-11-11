---
default: minor
---

# feat: state machine override, input reset and input listing in the C API

Adds `dotlottie_state_machine_override_current_state`, `dotlottie_state_machine_reset_input`, `dotlottie_state_machine_get_inputs_count`, `dotlottie_state_machine_get_input_name` and `dotlottie_state_machine_get_input_type` to the C API. The WASM bindings already had these, so native wrappers can now offer them too.
