// _DOTLOTTIE_BRIDGE_JS is a global object which will hold the implementation of the bridge functions
mergeInto(LibraryManager.library, {
  _emscripten_get_now: function () {
    return Date.now();
  }
});
