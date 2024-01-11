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
        .function("setFrame", &DotLottiePlayer::set_frame)
        .function("currentFrame", &DotLottiePlayer::current_frame)
        .function("duration", &DotLottiePlayer::duration)
        .function("totalFrames", &DotLottiePlayer::total_frames)
        .function("loadAnimationData", &DotLottiePlayer::load_animation_data)
        .function("getBuffer", &getBuffer);
}