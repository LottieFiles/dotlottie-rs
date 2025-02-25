#include "dotlottie_player.hpp"
#include <emscripten/bind.h>
#include <emscripten/emscripten.h>

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

    // Register std::vector<float> as VectorFloat for the Config::segment field
    register_vector<float>("VectorFloat");
    register_vector<Marker>("VectorMarker");
    register_vector<std::string>("VectorString");

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

    enum_<OpenUrlMode>("OpenUrlMode")
        .value("Deny", OpenUrlMode::kDeny)
        .value("Interaction", OpenUrlMode::kInteraction)
        .value("Allow", OpenUrlMode::kAllow);

    value_object<OpenUrl>("OpenUrl")
        .field("mode", &OpenUrl::mode)
        .field("whitelist", &OpenUrl::whitelist);

    function("createDefaultOpenURL", &create_default_open_url);

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
        .field("marker", &Config::marker)
        .field("themeId", &Config::theme_id)
        .field("stateMachineId", &Config::state_machine_id);

    function("createDefaultConfig", &create_default_config);
    function("transformThemeToLottieSlots", &transform_theme_to_lottie_slots);

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
        .function("isComplete", &DotLottiePlayer::is_complete)
        .function("setTheme", &DotLottiePlayer::set_theme)
        .function("setThemeData", &DotLottiePlayer::set_theme_data)
        .function("resetTheme", &DotLottiePlayer::reset_theme)
        .function("setSlots", &DotLottiePlayer::set_slots)
        .function("markers", &DotLottiePlayer::markers)
        .function("activeAnimationId", &DotLottiePlayer::active_animation_id)
        .function("activeThemeId", &DotLottiePlayer::active_theme_id)
        .function("setViewport", &DotLottiePlayer::set_viewport)
        .function("segmentDuration", &DotLottiePlayer::segment_duration)
        .function("animationSize", &DotLottiePlayer::animation_size)

        .function("stateMachineLoad", &DotLottiePlayer::state_machine_load)
        .function("stateMachineStart", &DotLottiePlayer::state_machine_start)
        .function("stateMachineStop", &DotLottiePlayer::state_machine_stop)
        .function("stateMachineFrameworkSetup", &DotLottiePlayer::state_machine_framework_setup)
        .function("stateMachineLoadData", &DotLottiePlayer::state_machine_load_data)
        .function("stateMachineFireEvent", &DotLottiePlayer::state_machine_fire_event)
        .function("stateMachineSetNumericInput", &DotLottiePlayer::state_machine_set_numeric_input)
        .function("stateMachineSetStringInput", &DotLottiePlayer::state_machine_set_string_input)
        .function("stateMachineSetBooleanInput", &DotLottiePlayer::state_machine_set_boolean_input)
        .function("stateMachineGetNumericInput", &DotLottiePlayer::state_machine_get_numeric_input)
        .function("stateMachineGetStringInput", &DotLottiePlayer::state_machine_get_string_input)
        .function("stateMachineGetBooleanInput", &DotLottiePlayer::state_machine_get_boolean_input)
        .function("getLayerBounds", &DotLottiePlayer::get_layer_bounds)
        .function("getStateMachine", &DotLottiePlayer::get_state_machine)
        .function("activeStateMachineId", &DotLottiePlayer::active_state_machine_id)
        .function("stateMachineCurrentState", &DotLottiePlayer::state_machine_current_state)
        .function("stateMachinePostClickEvent", &DotLottiePlayer::state_machine_post_click_event)
        .function("stateMachinePostPointerDownEvent", &DotLottiePlayer::state_machine_post_pointer_down_event)
        .function("stateMachinePostPointerUpEvent", &DotLottiePlayer::state_machine_post_pointer_up_event)
        .function("stateMachinePostPointerMoveEvent", &DotLottiePlayer::state_machine_post_pointer_move_event)
        .function("stateMachinePostPointerEnterEvent", &DotLottiePlayer::state_machine_post_pointer_enter_event)
        .function("stateMachinePostPointerExitEvent", &DotLottiePlayer::state_machine_post_pointer_exit_event)
        .function("stateMachineOverrideCurrentState", &DotLottiePlayer::state_machine_override_current_state)
        .function("stateMachineStatus", &DotLottiePlayer::state_machine_status)
        .function("instanceId", &DotLottiePlayer::instance_id);
}