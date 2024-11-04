#include "dotlottie_player.hpp"
#include <emscripten/bind.h>
#include <emscripten/emscripten.h>

using namespace emscripten;
using namespace dotlottie_player;

extern "C"
{
    /*
        This is a workaround as instant crate expects a _ prefix for the emscripten_get_now function
        https://github.com/sebcrozet/instant/issues/35
    */
    double _emscripten_get_now()
    {
        return emscripten_get_now();
    }
}

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

    // Register std::vector<float> as VectorFloat for the Config::segment field
    register_vector<float>("VectorFloat");
    register_vector<Marker>("VectorMarker");
    register_vector<std::string>("VectorString");
    // register_vector<ManifestTheme>("VectorManifestTheme");
    // register_vector<ManifestAnimation>("VectorManifestAnimation");

    enum_<Mode>("Mode")
        .value("Forward", Mode::kForward)
        .value("Reverse", Mode::kReverse)
        .value("Bounce", Mode::kBounce)
        .value("ReverseBounce", Mode::kReverseBounce);

    enum_<Fit>("Fit")
        .value("Contain", Fit::kContain)
        .value("Cover", Fit::kCover)
        .value("Fill", Fit::kFill)
        .value("FitWidth", Fit::kFitWidth)
        .value("FitHeight", Fit::kFitHeight)
        .value("None", Fit::kNone);

    value_object<Layout>("Layout")
        .field("fit", &Layout::fit)
        .field("align", &Layout::align);

    function("createDefaultLayout", &create_default_layout);

    value_object<Marker>("Marker")
        .field("name", &Marker::name)
        .field("time", &Marker::time)
        .field("duration", &Marker::duration);

    value_object<Config>("Config")
        .field("autoplay", &Config::autoplay)
        .field("loopAnimation", &Config::loop_animation)
        .field("mode", &Config::mode)
        .field("speed", &Config::speed)
        .field("useFrameInterpolation", &Config::use_frame_interpolation)
        .field("segment", &Config::segment)
        .field("backgroundColor", &Config::background_color)
        .field("layout", &Config::layout)
        .field("marker", &Config::marker);

    function("createDefaultConfig", &create_default_config);
    function("transformThemeToLottieSlots", &transform_theme_to_lottie_slots);

    // value_object<ManifestTheme>("ManifestTheme")
    //     .field("id", &ManifestTheme::id)
    //     .field("animations", &ManifestTheme::animations);

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

    // class_<Observer>("Observer")
    //     .smart_ptr<std::shared_ptr<Observer>>("Observer")
    //     .function("onFrame", &Observer::on_frame)
    //     .function("onLoad", &Observer::on_load)
    //     .function("onLoop", &Observer::on_loop)
    //     .function("onPause", &Observer::on_pause)
    //     .function("onPlay", &Observer::on_play)
    //     .function("onRender", &Observer::on_render)
    //     .function("onComplete", &Observer::on_complete)
    //     .function("onStop", &Observer::on_stop);

    // class_<StateMachineObserver>("StateMachineObserver")
    //     .smart_ptr<std::shared_ptr<StateMachineObserver>>("StateMachineObserver")
    //     .function("OnTransition", &StateMachineObserver::on_transition);
    //     .function("onStateEntered", &StateMachineObserver::on_state_entered);
    //     .function("onStateExit", &StateMachineObserver::on_state_exit);

    class_<DotLottiePlayer>("DotLottiePlayer")
        .smart_ptr<std::shared_ptr<DotLottiePlayer>>("DotLottiePlayer")
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
        // .function("manifest", &DotLottiePlayer::manifest)
        .function("manifestString", &DotLottiePlayer::manifest_string)
        .function("loopCount", &DotLottiePlayer::loop_count)
        .function("pause", &DotLottiePlayer::pause)
        .function("play", &DotLottiePlayer::play)
        .function("render", &DotLottiePlayer::render)
        .function("requestFrame", &DotLottiePlayer::request_frame)
        .function("resize", &DotLottiePlayer::resize)
        .function("setConfig", &DotLottiePlayer::set_config)
        .function("setFrame", &DotLottiePlayer::set_frame)
        .function("seek", &DotLottiePlayer::seek)
        .function("stop", &DotLottiePlayer::stop)
        .function("totalFrames", &DotLottiePlayer::total_frames)
        // .function("subscribe", &DotLottiePlayer::subscribe)
        // .function("unsubscribe", &DotLottiePlayer::unsubscribe)
        .function("isComplete", &DotLottiePlayer::is_complete)
        .function("loadTheme", &DotLottiePlayer::load_theme)
        .function("loadThemeData", &DotLottiePlayer::load_theme_data)
        .function("setSlots", &DotLottiePlayer::set_slots)
        .function("markers", &DotLottiePlayer::markers)
        .function("activeAnimationId", &DotLottiePlayer::active_animation_id)
        .function("activeThemeId", &DotLottiePlayer::active_theme_id)
        .function("setViewport", &DotLottiePlayer::set_viewport)
        .function("segmentDuration", &DotLottiePlayer::segment_duration)
        .function("animationSize", &DotLottiePlayer::animation_size)

        .function("loadStateMachine", &DotLottiePlayer::load_state_machine)
        .function("startStateMachine", &DotLottiePlayer::start_state_machine)
        .function("stopStateMachine", &DotLottiePlayer::stop_state_machine)
        .function("stateMachineFrameworkSetup", &DotLottiePlayer::state_machine_framework_setup)
        .function("setStateMachineNumericContext", &DotLottiePlayer::set_state_machine_numeric_context)
        .function("setStateMachineStringContext", &DotLottiePlayer::set_state_machine_string_context)
        .function("setStateMachineBooleanContext", &DotLottiePlayer::set_state_machine_boolean_context)
        .function("loadStateMachineData", &DotLottiePlayer::load_state_machine_data)
        .function("getLayerBounds", &DotLottiePlayer::get_layer_bounds)
        .function("postBoolEvent", &DotLottiePlayer::post_bool_event)
        .function("postStringEvent", &DotLottiePlayer::post_string_event)
        .function("postNumericEvent", &DotLottiePlayer::post_numeric_event)
        .function("postPointerDownEvent", &DotLottiePlayer::post_pointer_down_event)
        .function("postPointerUpEvent", &DotLottiePlayer::post_pointer_up_event)
        .function("postPointerMoveEvent", &DotLottiePlayer::post_pointer_move_event)
        .function("postPointerEnterEvent", &DotLottiePlayer::post_pointer_enter_event)
        .function("postPointerExitEvent", &DotLottiePlayer::post_pointer_exit_event)
        .function("postSetNumericContext", &DotLottiePlayer::post_set_numeric_context);
    // .function("state_machine_subscribe", &DotLottiePlayer::state_machine_subscribe)
    // .function("state_machine_unsubscribe", &DotLottiePlayer::state_machine_unsubscribe)
}
