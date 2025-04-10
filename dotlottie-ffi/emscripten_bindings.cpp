#include "dotlottie_player.hpp"
#include <emscripten/bind.h>
#include <emscripten/emscripten.h>
#include <optional>

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

struct ObserverWrapper : public wrapper<Observer>
{
    EMSCRIPTEN_WRAPPER(ObserverWrapper);
    void on_complete()
    {
        return call<void>("on_complete");
    }
    void on_frame(float frame_no)
    {
        return call<void>("on_frame", frame_no);
    }
    void on_load()
    {
        return call<void>("on_load");
    }
    void on_load_error()
    {
        return call<void>("on_load_error");
    }
    void on_loop(uint32_t loop_count)
    {
        return call<void>("on_loop", loop_count);
    }
    void on_pause()
    {
        return call<void>("on_pause");
    }
    void on_play()
    {
        return call<void>("on_play");
    }
    void on_render(float frame_no)
    {
        return call<void>("on_render", frame_no);
    }
    void on_stop()
    {
        return call<void>("on_stop");
    }
};

struct StateMachineObserverWrapper : public wrapper<StateMachineObserver>
{
    EMSCRIPTEN_WRAPPER(StateMachineObserverWrapper);
    void on_start()
    {
        return call<void>("on_start");
    }
    void on_stop()
    {
        return call<void>("on_stop");
    }
    void on_transition(const std::string &previous_state, const std::string &new_state)
    {
        return call<void>("on_transition", previous_state, new_state);
    }
    void on_state_entered(const std::string &entering_state)
    {
        return call<void>("on_state_entered", entering_state);
    }
    void on_state_exit(const std::string &leaving_state)
    {
        return call<void>("on_state_exit", leaving_state);
    }
    void on_custom_event(const std::string &event)
    {
        return call<void>("on_custom_event", event);
    }
    void on_string_input_value_change(const std::string &input_name, const std::string &old_value, const std::string &new_value)
    {
        return call<void>("on_string_input_value_change", input_name, old_value, new_value);
    }
    void on_numeric_input_value_change(const std::string &input_name, float old_value, float new_value)
    {
        return call<void>("on_numeric_input_value_change", input_name, old_value, new_value);
    }
    void on_boolean_input_value_change(const std::string &input_name, bool old_value, bool new_value)
    {
        return call<void>("on_boolean_input_value_change", input_name, old_value, new_value);
    }
    void on_input_fired(const std::string &input_name)
    {
        return call<void>("on_input_fired", input_name);
    }
    void on_error(const std::string &error)
    {
        return call<void>("on_error", error);
    }
};

std::shared_ptr<Observer> subscribe(DotLottiePlayer &player, Observer *observer)
{
    // Create a shared pointer to the observer
    std::shared_ptr<Observer> ob = std::shared_ptr<Observer>(observer);    

    player.subscribe(ob);

    return ob;
}

void unsubscribe(DotLottiePlayer &player, std::shared_ptr<Observer> observer)
{
    player.unsubscribe(observer);
}

std::shared_ptr<StateMachineObserver> stateMachineSubscribe(DotLottiePlayer &player, StateMachineObserver *observer)
{
    std::shared_ptr<StateMachineObserver> ob = std::shared_ptr<StateMachineObserver>(observer);

    player.state_machine_subscribe(ob);

    return ob;
}

void stateMachineUnsubscribe(DotLottiePlayer &player,std::shared_ptr<StateMachineObserver> observer)
{
    player.state_machine_unsubscribe(observer);
}

std::shared_ptr<StateMachineObserver> stateMachineFrameworkSubscribe(DotLottiePlayer &player, StateMachineObserver *observer)
{
    std::shared_ptr<StateMachineObserver> ob = std::shared_ptr<StateMachineObserver>(observer);

    player.state_machine_framework_subscribe(ob);

    return ob;
}

void stateMachineFrameworkUnsubscribe(DotLottiePlayer &player,std::shared_ptr<StateMachineObserver> observer)
{
    player.state_machine_framework_unsubscribe(observer);
}

EMSCRIPTEN_BINDINGS(DotLottiePlayer)
{

    // Register std::vector<float> as VectorFloat for the Config::segment field
    register_vector<float>("VectorFloat");
    // Then register the optional type for the vector - without a name parameter
    register_optional<std::vector<float>>();
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

    class_<Observer>("Observer")
        .smart_ptr<std::shared_ptr<Observer>>("Observer")
        .function("on_load", &Observer::on_load, pure_virtual())
        .function("on_load_error", &Observer::on_load_error, pure_virtual())
        .function("on_play", &Observer::on_play, pure_virtual())
        .function("on_pause", &Observer::on_pause, pure_virtual())
        .function("on_stop", &Observer::on_stop, pure_virtual())
        .function("on_frame", &Observer::on_frame, pure_virtual())
        .function("on_render", &Observer::on_render, pure_virtual())
        .function("on_loop", &Observer::on_loop, pure_virtual())
        .function("on_complete", &Observer::on_complete, pure_virtual())
        .allow_subclass<ObserverWrapper>("ObserverWrapper");

    class_<StateMachineObserver>("StateMachineObserver")
        .smart_ptr<std::shared_ptr<StateMachineObserver>>("StateMachineObserver")
        .function("on_start", &StateMachineObserver::on_start, pure_virtual())
        .function("on_stop", &StateMachineObserver::on_stop, pure_virtual())
        .function("on_transition", &StateMachineObserver::on_transition, pure_virtual())
        .function("on_state_entered", &StateMachineObserver::on_state_entered, pure_virtual())
        .function("on_state_exit", &StateMachineObserver::on_state_exit, pure_virtual())
        .function("on_custom_event", &StateMachineObserver::on_custom_event, pure_virtual())
        .function("on_string_input_value_change", &StateMachineObserver::on_string_input_value_change, pure_virtual())
        .function("on_numeric_input_value_change", &StateMachineObserver::on_numeric_input_value_change, pure_virtual())
        .function("on_boolean_input_value_change", &StateMachineObserver::on_boolean_input_value_change, pure_virtual())
        .function("on_input_fired", &StateMachineObserver::on_input_fired, pure_virtual())
        .function("on_error", &StateMachineObserver::on_error, pure_virtual())
        .allow_subclass<StateMachineObserverWrapper>("StateMachineObserverWrapper");

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
        .function("subscribe", &subscribe, allow_raw_pointers())
        .function("unsubscribe", &unsubscribe, allow_raw_pointers())

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
        .function("intersect", &DotLottiePlayer::intersect)
        .function("getLayerBounds", &DotLottiePlayer::get_layer_bounds)
        .function("tick", &DotLottiePlayer::tick)
        .function("tween", &DotLottiePlayer::tween)
        .function("tweenStop", &DotLottiePlayer::tween_stop)
        .function("tweenToMarker", &DotLottiePlayer::tween_to_marker)
        .function("isTweening", &DotLottiePlayer::is_tweening)
        .function("tweenUpdate", &DotLottiePlayer::tween_update)

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
        .function("stateMachineSubscribe", &stateMachineSubscribe, allow_raw_pointers())
        .function("stateMachineUnsubscribe", &stateMachineUnsubscribe, allow_raw_pointers())
        .function("stateMachineFrameworkSubscribe", &stateMachineFrameworkSubscribe, allow_raw_pointers())
        .function("stateMachineFrameworkUnsubscribe", &stateMachineFrameworkUnsubscribe, allow_raw_pointers())
        .function("instanceId", &DotLottiePlayer::instance_id);
}