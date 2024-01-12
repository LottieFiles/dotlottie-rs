#include "dotlottie_player.hpp"
#include <emscripten/bind.h>

using namespace emscripten;
using namespace dotlottie_player;

val getBuffer(DotLottiePlayer &player)
{
    auto buffer_ptr = player.buffer_ptr();
    auto buffer_len = player.buffer_len();
    return val(typed_memory_view(buffer_len, reinterpret_cast<uint8_t *>(buffer_ptr)));
}
