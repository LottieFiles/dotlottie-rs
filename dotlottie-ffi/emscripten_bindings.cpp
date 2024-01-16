#include "dotlottie_player.hpp"
#include <emscripten/bind.h>

using namespace emscripten;
using namespace dotlottie_player;

val buffer(DotLottiePlayer &player)
{
    auto buffer_ptr = player.buffer_ptr();
    auto buffer_len = player.buffer_len();
    return val(typed_memory_view(buffer_len, reinterpret_cast<uint8_t *>(buffer_ptr)));
}

EMSCRIPTEN_BINDINGS(DotLottiePlayer)
{
    enum_<Mode>("Mode")
        .value("Forward", Mode::FORWARD)
        .value("Reverse", Mode::REVERSE)
        .value("Bounce", Mode::BOUNCE)
        .value("ReverseBounce", Mode::REVERSE_BOUNCE);

    value_object<Config>("Config")
        .field("autoplay", &Config::autoplay)
        .field("loop_animation", &Config::loop_animation)
        .field("mode", &Config::mode)
        .field("speed", &Config::speed)
        .field("use_frame_interpolation", &Config::use_frame_interpolation);

    class_<DotLottiePlayer>("DotLottiePlayer")
        .constructor(&DotLottiePlayer::init, allow_raw_pointers())
        .function("buffer_len", &DotLottiePlayer::buffer_len)
        .function("buffer_ptr", &DotLottiePlayer::buffer_ptr)
        .function("buffer", &buffer)
        .function("clear", &DotLottiePlayer::clear)
        .function("config", &DotLottiePlayer::config)
        .function("current_frame", &DotLottiePlayer::current_frame)
        .function("duration", &DotLottiePlayer::duration)
        .function("is_loaded", &DotLottiePlayer::is_loaded)
        .function("is_paused", &DotLottiePlayer::is_paused)
        .function("is_playing", &DotLottiePlayer::is_playing)
        .function("is_stopped", &DotLottiePlayer::is_stopped)
        .function("load_animation_data", &DotLottiePlayer::load_animation_data, allow_raw_pointers())
        .function("load_animation_path", &DotLottiePlayer::load_animation_path, allow_raw_pointers())
        .function("loop_count", &DotLottiePlayer::loop_count)
        .function("pause", &DotLottiePlayer::pause)
        .function("play", &DotLottiePlayer::play)
        .function("render", &DotLottiePlayer::render)
        .function("request_frame", &DotLottiePlayer::request_frame)
        .function("resize", &DotLottiePlayer::resize)
        .function("set_config", &DotLottiePlayer::set_config)
        .function("set_frame", &DotLottiePlayer::set_frame)
        .function("stop", &DotLottiePlayer::stop)
        .function("total_frames", &DotLottiePlayer::total_frames);
}