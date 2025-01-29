// _DOTLOTTIE_BRIDGE_JS is a global object which will hold the implementation of the bridge functions
mergeInto(LibraryManager.library, {
  _emscripten_get_now: function () {
    return Date.now();
  },
  observer_on_load(dotlottie_instance_id) {
    if (Module.dotlottieBridge && Module.dotlottieBridge.observer_on_load) {
      Module.dotlottieBridge.observer_on_load(dotlottie_instance_id);
    }
  },
  observer_on_load_error(dotlottie_instance_id) {
    if (Module.dotlottieBridge && Module.dotlottieBridge.observer_on_load_error) {
      Module.dotlottieBridge.observer_on_load_error(dotlottie_instance_id);
    }
  },
  observer_on_play(dotlottie_instance_id) {
    if (Module.dotlottieBridge && Module.dotlottieBridge.observer_on_load_error) {
      Module.dotlottieBridge.observer_on_play(dotlottie_instance_id);
    }
  },
  observer_on_pause(dotlottie_instance_id) {
    if (Module.dotlottieBridge && Module.dotlottieBridge.observer_on_load_error) {
      Module.dotlottieBridge.observer_on_pause(dotlottie_instance_id);
    }
  },
  observer_on_stop(dotlottie_instance_id) {
    if (Module.dotlottieBridge && Module.dotlottieBridge.observer_on_load_error) {
      Module.dotlottieBridge.observer_on_stop(dotlottie_instance_id);
    }
  },
  observer_on_frame(dotlottie_instance_id, frame_no) {
    if (Module.dotlottieBridge && Module.dotlottieBridge.observer_on_load_error) {
      Module.dotlottieBridge.observer_on_frame(dotlottie_instance_id, frame_no);
    }
  },
  observer_on_render(dotlottie_instance_id, frame_no) {
    if (Module.dotlottieBridge && Module.dotlottieBridge.observer_on_load_error) {
      Module.dotlottieBridge.observer_on_render(dotlottie_instance_id, frame_no);
    }
  },
  observer_on_loop(dotlottie_instance_id, loop_count) {
    if (Module.dotlottieBridge && Module.dotlottieBridge.observer_on_load_error) {
      Module.dotlottieBridge.observer_on_loop(dotlottie_instance_id, loop_count);
    }
  },
  observer_on_complete(dotlottie_instance_id) {
    if (Module.dotlottieBridge && Module.dotlottieBridge.observer_on_load_error) {
      Module.dotlottieBridge.observer_on_complete(dotlottie_instance_id);
    }
  },
  state_machine_observer_on_transition: function (dotlottie_instance_id, prev_state_ptr, prev_state_len, new_state_ptr, new_state_len) {
    const previousState = UTF8ToString(prev_state_ptr, prev_state_len);
    const newState = UTF8ToString(new_state_ptr, new_state_len);

    if (Module.dotlottieBridge && Module.dotlottieBridge.state_machine_observer_on_transition) {
      Module.dotlottieBridge.state_machine_observer_on_transition(
        dotlottie_instance_id,
        previousState,
        newState
      );
    }
  },
  state_machine_observer_on_state_entered: function (dotlottie_instance_id, state_ptr, state_len) {
    const state = UTF8ToString(state_ptr, state_len);

    if (Module.dotlottieBridge && Module.dotlottieBridge.state_machine_observer_on_state_entered) {
      Module.dotlottieBridge.state_machine_observer_on_state_entered(
        dotlottie_instance_id,
        state
      );
    }
  },

  state_machine_observer_on_state_exit: function (dotlottie_instance_id, state_ptr, state_len) {
    const state = UTF8ToString(state_ptr, state_len);

    if (Module.dotlottieBridge && Module.dotlottieBridge.state_machine_observer_on_state_entered) {
      Module.dotlottieBridge.state_machine_observer_on_state_exit(
        dotlottie_instance_id,
        state
      );
    }
  },

  state_machine_observer_on_custom_event: function (dotlottie_instance_id, message_ptr, message_len) {
    const message = UTF8ToString(message_ptr, message_len);

    if (Module.dotlottieBridge && Module.dotlottieBridge.state_machine_observer_on_state_entered) {
      Module.dotlottieBridge.state_machine_observer_on_state_entered(
        dotlottie_instance_id,
        message
      );
    }
  },

});
