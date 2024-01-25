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

bool load_dotlottie_data(DotLottiePlayer &player, std::string data, uint32_t width, uint32_t height)
{
    std::vector<char> data_vector(data.begin(), data.end());

    return player.load_dotlottie_data(data_vector, width, height);
}

EMSCRIPTEN_BINDINGS(DotLottiePlayer)
{

    // Register std::vector<float> as VectorFloat for the Config::segments field
    register_vector<float>("VectorFloat");

    enum_<Mode>("Mode")
        .value("Forward", Mode::FORWARD)
        .value("Reverse", Mode::REVERSE)
        .value("Bounce", Mode::BOUNCE)
        .value("ReverseBounce", Mode::REVERSE_BOUNCE);

    value_object<Config>("Config")
        .field("autoplay", &Config::autoplay)
        .field("loopAnimation", &Config::loop_animation)
        .field("mode", &Config::mode)
        .field("speed", &Config::speed)
        .field("useFrameInterpolation", &Config::use_frame_interpolation)
        .field("segments", &Config::segments)
        .field("backgroundColor", &Config::background_color);

    // value_object<ManifestTheme>("ManifestTheme")
    //     .field("id", &ManifestTheme::id)
    //     .field("values", &ManifestTheme::values);

    // value_object<ManifestThemes>("ManifestThemes")
    //     .field("value", &ManifestThemes::value);

    // value_object<ManifestAnimation>("ManifestAnimation")
    //     .field("autoplay", &ManifestAnimation::autoplay)
    //     .field("defaultTheme", &ManifestAnimation::default_theme)
    //     .field("direction", &ManifestAnimation::direction)
    //     .field("hover", &ManifestAnimation::hover)
    //     .field("id", &ManifestAnimation::id)
    //     .field("intermission", &ManifestAnimation::intermission)
    //     .field("loop", &ManifestAnimation::loop)
    //     .field("loop_count", &ManifestAnimation::loop_count)
    //     .field("playMode", &ManifestAnimation::play_mode)
    //     .field("speed", &ManifestAnimation::speed)
    //     .field("themeColor", &ManifestAnimation::theme_color);

    // value_object<Manifest>("Manifest")
    //     .field("active_animation_id", &Manifest::active_animation_id)
    //     .field("animations", &Manifest::animations)
    //     .field("author", &Manifest::author)
    //     .field("description", &Manifest::description)
    //     .field("generator", &Manifest::generator)
    //     .field("keywords", &Manifest::keywords)
    //     .field("revision", &Manifest::revision)
    //     .field("themes", &Manifest::themes)
    //     .field("states", &Manifest::states)
    //     .field("version", &Manifest::version);

    class_<DotLottiePlayer>("DotLottiePlayer")
        .constructor(&DotLottiePlayer::init, allow_raw_pointers())
        .function("buffer", &buffer)
        .function("clear", &DotLottiePlayer::clear)
        .function("config", &DotLottiePlayer::config)
        .function("currentFrame", &DotLottiePlayer::current_frame)
        .function("duration", &DotLottiePlayer::duration)
        .function("isLoaded", &DotLottiePlayer::is_loaded)
        .function("isPaused", &DotLottiePlayer::is_paused)
        .function("isPlaying", &DotLottiePlayer::is_playing)
        .function("isStopped", &DotLottiePlayer::is_stopped)
        .function("loadAnimationData", &DotLottiePlayer::load_animation_data, allow_raw_pointers())
        .function("loadAnimationPath", &DotLottiePlayer::load_animation_path, allow_raw_pointers())
        .function("loadDotLottieData", &load_dotlottie_data, allow_raw_pointers())
        .function("loadAnimation", &DotLottiePlayer::load_animation, allow_raw_pointers())
        // FIXME: .function("manifest", &DotLottiePlayer::manifest)
        .function("manifestString", &DotLottiePlayer::manifest_string)
        .function("loopCount", &DotLottiePlayer::loop_count)
        .function("pause", &DotLottiePlayer::pause)
        .function("play", &DotLottiePlayer::play)
        .function("render", &DotLottiePlayer::render)
        .function("requestFrame", &DotLottiePlayer::request_frame)
        .function("resize", &DotLottiePlayer::resize)
        .function("setConfig", &DotLottiePlayer::set_config)
        .function("setFrame", &DotLottiePlayer::set_frame)
        .function("stop", &DotLottiePlayer::stop)
        .function("totalFrames", &DotLottiePlayer::total_frames);
}