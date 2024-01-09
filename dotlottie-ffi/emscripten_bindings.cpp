#include "dotlottie_player.hpp"
#include <emscripten/bind.h>

using namespace emscripten;
using namespace dotlottie_player;

val getBuffer(DotLottiePlayer &player)
{
    auto buffer_ptr = player.get_buffer();
    auto buffer_size = player.get_buffer_size();
    return val(typed_memory_view(buffer_size, reinterpret_cast<uint8_t *>(buffer_ptr)));
}

EMSCRIPTEN_BINDINGS(DotLottiePlayer)
{
    class_<DotLottiePlayer>("DotLottiePlayer")
        .constructor(&DotLottiePlayer::init)
        .function("frame", &DotLottiePlayer::frame)
        .function("getCurrentFrame", &DotLottiePlayer::get_current_frame)
        .function("getDuration", &DotLottiePlayer::get_duration)
        .function("getTotalFrame", &DotLottiePlayer::get_total_frame)
        .function("loadAnimation", &DotLottiePlayer::load_animation)
        .function("getBuffer", &getBuffer);
}