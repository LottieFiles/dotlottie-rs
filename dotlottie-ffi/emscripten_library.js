// _DOTLOTTIE_BRIDGE_JS is a global object which will hold the implementation of the bridge functions
mergeInto(LibraryManager.library, {
  _emscripten_get_now: function () {
    return Date.now();
  },
  observer_on_load(dotlottie_instance_id) {
    _DOTLOTTIE_BRIDGE_JS.observer_on_load(dotlottie_instance_id);
  },
  observer_on_load_error(dotlottie_instance_id) {
    _DOTLOTTIE_BRIDGE_JS.observer_on_load_error(dotlottie_instance_id);
  },
  observer_on_play(dotlottie_instance_id) {
    _DOTLOTTIE_BRIDGE_JS.observer_on_play(dotlottie_instance_id);
  },
  observer_on_pause(dotlottie_instance_id) {
    _DOTLOTTIE_BRIDGE_JS.observer_on_pause(dotlottie_instance_id);
  },
  observer_on_stop(dotlottie_instance_id) {
    _DOTLOTTIE_BRIDGE_JS.observer_on_stop(dotlottie_instance_id);
  },
  observer_on_frame(dotlottie_instance_id, frame_no) {
    _DOTLOTTIE_BRIDGE_JS.observer_on_frame(dotlottie_instance_id, frame_no);
  },
  observer_on_render(dotlottie_instance_id, frame_no) {
    _DOTLOTTIE_BRIDGE_JS.observer_on_render(dotlottie_instance_id, frame_no);
  },
  observer_on_loop(dotlottie_instance_id, loop_count) {
    _DOTLOTTIE_BRIDGE_JS.observer_on_loop(dotlottie_instance_id, loop_count);
  },
  observer_on_complete(dotlottie_instance_id) {
    _DOTLOTTIE_BRIDGE_JS.observer_on_complete(dotlottie_instance_id);
  },
});
