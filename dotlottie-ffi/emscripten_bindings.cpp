#include "dotlottie_player.hpp"
#include <emscripten/bind.h>
#include <emscripten/emscripten.h>
#include <optional>
#include <functional>

using namespace emscripten;
using namespace dotlottie_player;

val buffer(DotLottiePlayer &player)
{
    auto buffer_ptr = (uint32_t *)player.buffer_ptr();
    auto buffer_len = player.buffer_len() * sizeof(uint32_t);
    return val(typed_memory_view(buffer_len, reinterpret_cast<uint8_t *>(buffer_ptr)));
}

bool load_dotlottie_data(DotLottiePlayer &player, std::string data, uint32_t width, uint32_t height)
{
    std::vector<char> data_vector(data.begin(), data.end());

    return player.load_dotlottie_data(data_vector, width, height);
}

struct ObserverCallbacks {
    std::function<void()> on_complete;
    std::function<void()> on_load;
    std::function<void()> on_load_error;
    std::function<void()> on_play;
    std::function<void()> on_pause;
    std::function<void()> on_stop;
    std::function<void(float)> on_frame;
    std::function<void(float)> on_render;
    std::function<void(uint32_t)> on_loop;
};

class CallbackObserver : public Observer {
public:
    CallbackObserver() = default;
    void setOnComplete(val cb) { 
        callbacks_.on_complete = [cb]() { if (cb != val::undefined()) cb(); };
    }
    void setOnLoad(val cb) { 
        callbacks_.on_load = [cb]() { if (cb != val::undefined()) cb(); };
    }
    void setOnLoadError(val cb) { 
        callbacks_.on_load_error = [cb]() { if (cb != val::undefined()) cb(); };
    }
    void setOnPlay(val cb) { 
        callbacks_.on_play = [cb]() { if (cb != val::undefined()) cb(); };
    }
    void setOnPause(val cb) { 
        callbacks_.on_pause = [cb]() { if (cb != val::undefined()) cb(); };
    }
    void setOnStop(val cb) { 
        callbacks_.on_stop = [cb]() { if (cb != val::undefined()) cb(); };
    }
    void setOnFrame(val cb) { 
        callbacks_.on_frame = [cb](float frame_no) { if (cb != val::undefined()) cb(frame_no); };
    }
    void setOnRender(val cb) { 
        callbacks_.on_render = [cb](float frame_no) { if (cb != val::undefined()) cb(frame_no); };
    }
    void setOnLoop(val cb) { 
        callbacks_.on_loop = [cb](uint32_t loop_count) { if (cb != val::undefined()) cb(loop_count); };
    }

    void on_complete() override { if (callbacks_.on_complete) callbacks_.on_complete(); }
    void on_load() override { if (callbacks_.on_load) callbacks_.on_load(); }
    void on_load_error() override { if (callbacks_.on_load_error) callbacks_.on_load_error(); }
    void on_play() override { if (callbacks_.on_play) callbacks_.on_play(); }
    void on_pause() override { if (callbacks_.on_pause) callbacks_.on_pause(); }
    void on_stop() override { if (callbacks_.on_stop) callbacks_.on_stop(); }
    void on_frame(float frame_no) override { if (callbacks_.on_frame) callbacks_.on_frame(frame_no); }
    void on_render(float frame_no) override { if (callbacks_.on_render) callbacks_.on_render(frame_no); }
    void on_loop(uint32_t loop_count) override { if (callbacks_.on_loop) callbacks_.on_loop(loop_count); }
private:
    ObserverCallbacks callbacks_;
};

struct StateMachineInternalObserverCallbacks {
    std::function<void(const std::string&)> on_message;
};

class CallbackStateMachineInternalObserver : public StateMachineInternalObserver {
public:
    CallbackStateMachineInternalObserver() = default;

    void setOnMessage(val cb) { 
        callbacks_.on_message = [cb](const std::string& message) { 
            if (cb != val::undefined()) cb(message); 
        };
    }

    void on_message(const std::string &message) override { if (callbacks_.on_message) callbacks_.on_message(message); }
private:
    StateMachineInternalObserverCallbacks callbacks_;
};

struct StateMachineObserverCallbacks {
    std::function<void()> on_start;
    std::function<void()> on_stop;
    std::function<void(const std::string&, const std::string&)> on_transition;
    std::function<void(const std::string&)> on_state_entered;
    std::function<void(const std::string&)> on_state_exit;
    std::function<void(const std::string&)> on_custom_event;
    std::function<void(const std::string&, const std::string&, const std::string&)> on_string_input_value_change;
    std::function<void(const std::string&, float, float)> on_numeric_input_value_change;
    std::function<void(const std::string&, bool, bool)> on_boolean_input_value_change;
    std::function<void(const std::string&)> on_input_fired;
    std::function<void(const std::string&)> on_error;
};

class CallbackStateMachineObserver : public StateMachineObserver {
public:
    CallbackStateMachineObserver() = default;
    
    void setOnStart(val cb) { 
        callbacks_.on_start = [cb]() { if (cb != val::undefined()) cb(); };
    }
    void setOnStop(val cb) { 
        callbacks_.on_stop = [cb]() { if (cb != val::undefined()) cb(); };
    }
    void setOnTransition(val cb) { 
        callbacks_.on_transition = [cb](const std::string& prev, const std::string& next) { 
            if (cb != val::undefined()) cb(prev, next); 
        };
    }
    void setOnStateEntered(val cb) { 
        callbacks_.on_state_entered = [cb](const std::string& state) { 
            if (cb != val::undefined()) cb(state); 
        };
    }
    void setOnStateExit(val cb) { 
        callbacks_.on_state_exit = [cb](const std::string& state) { 
            if (cb != val::undefined()) cb(state); 
        };
    }
    void setOnCustomEvent(val cb) { 
        callbacks_.on_custom_event = [cb](const std::string& event) { 
            if (cb != val::undefined()) cb(event); 
        };
    }
    void setOnStringInputValueChange(val cb) { 
        callbacks_.on_string_input_value_change = [cb](const std::string& input, const std::string& oldv, const std::string& newv) { 
            if (cb != val::undefined()) cb(input, oldv, newv); 
        };
    }
    void setOnNumericInputValueChange(val cb) { 
        callbacks_.on_numeric_input_value_change = [cb](const std::string& input, float oldv, float newv) { 
            if (cb != val::undefined()) cb(input, oldv, newv); 
        };
    }
    void setOnBooleanInputValueChange(val cb) { 
        callbacks_.on_boolean_input_value_change = [cb](const std::string& input, bool oldv, bool newv) { 
            if (cb != val::undefined()) cb(input, oldv, newv); 
        };
    }
    void setOnInputFired(val cb) { 
        callbacks_.on_input_fired = [cb](const std::string& input) { 
            if (cb != val::undefined()) cb(input); 
        };
    }
    void setOnError(val cb) { 
        callbacks_.on_error = [cb](const std::string& err) { 
            if (cb != val::undefined()) cb(err); 
        };
    }

    void on_start() override { if (callbacks_.on_start) callbacks_.on_start(); }
    void on_stop() override { if (callbacks_.on_stop) callbacks_.on_stop(); }
    void on_transition(const std::string &prev, const std::string &next) override { if (callbacks_.on_transition) callbacks_.on_transition(prev, next); }
    void on_state_entered(const std::string &state) override { if (callbacks_.on_state_entered) callbacks_.on_state_entered(state); }
    void on_state_exit(const std::string &state) override { if (callbacks_.on_state_exit) callbacks_.on_state_exit(state); }
    void on_custom_event(const std::string &event) override { if (callbacks_.on_custom_event) callbacks_.on_custom_event(event); }
    void on_string_input_value_change(const std::string &input, const std::string &oldv, const std::string &newv) override { if (callbacks_.on_string_input_value_change) callbacks_.on_string_input_value_change(input, oldv, newv); }
    void on_numeric_input_value_change(const std::string &input, float oldv, float newv) override { if (callbacks_.on_numeric_input_value_change) callbacks_.on_numeric_input_value_change(input, oldv, newv); }
    void on_boolean_input_value_change(const std::string &input, bool oldv, bool newv) override { if (callbacks_.on_boolean_input_value_change) callbacks_.on_boolean_input_value_change(input, oldv, newv); }
    void on_input_fired(const std::string &input) override { if (callbacks_.on_input_fired) callbacks_.on_input_fired(input); }
    void on_error(const std::string &err) override { if (callbacks_.on_error) callbacks_.on_error(err); }
private:
    StateMachineObserverCallbacks callbacks_;
};

std::shared_ptr<Observer> subscribe(DotLottiePlayer &player, Observer* observer)
{
    // Create shared_ptr from raw pointer (without taking ownership)
    std::shared_ptr<Observer> shared_observer(observer, [](Observer*){});
    player.subscribe(shared_observer);
    return shared_observer;
}

void unsubscribe(DotLottiePlayer &player, std::shared_ptr<Observer> observer)
{
    player.unsubscribe(observer);
}

std::shared_ptr<StateMachineObserver> stateMachineSubscribe(DotLottiePlayer &player, StateMachineObserver* observer)
{
    // Create shared_ptr from raw pointer (without taking ownership)
    std::shared_ptr<StateMachineObserver> shared_observer(observer, [](StateMachineObserver*){});
    player.state_machine_subscribe(shared_observer);
    return shared_observer;
}

void stateMachineUnsubscribe(DotLottiePlayer &player, std::shared_ptr<StateMachineObserver> observer)
{
    player.state_machine_unsubscribe(observer);
}

std::shared_ptr<StateMachineInternalObserver> stateMachineInternalSubscribe(DotLottiePlayer &player, StateMachineInternalObserver* observer)
{
    // Create shared_ptr from raw pointer (without taking ownership)
    std::shared_ptr<StateMachineInternalObserver> shared_observer(observer, [](StateMachineInternalObserver*){});
    player.state_machine_internal_subscribe(shared_observer);
    return shared_observer;
}

void stateMachineInternalUnsubscribe(DotLottiePlayer &player, std::shared_ptr<StateMachineInternalObserver> observer)
{
    player.state_machine_internal_unsubscribe(observer);
}

EMSCRIPTEN_BINDINGS(observer_callbacks) {
    class_<CallbackObserver, base<Observer>>("CallbackObserver")
        .constructor<>()
        .function("setOnComplete", &CallbackObserver::setOnComplete)
        .function("setOnLoad", &CallbackObserver::setOnLoad)
        .function("setOnLoadError", &CallbackObserver::setOnLoadError)
        .function("setOnPlay", &CallbackObserver::setOnPlay)
        .function("setOnPause", &CallbackObserver::setOnPause)
        .function("setOnStop", &CallbackObserver::setOnStop)
        .function("setOnFrame", &CallbackObserver::setOnFrame)
        .function("setOnRender", &CallbackObserver::setOnRender)
        .function("setOnLoop", &CallbackObserver::setOnLoop);
}

EMSCRIPTEN_BINDINGS(state_machine_internal_observer_callbacks) {
    class_<CallbackStateMachineInternalObserver, base<StateMachineInternalObserver>>("CallbackStateMachineInternalObserver")
        .constructor<>()
        .function("setOnMessage", &CallbackStateMachineInternalObserver::setOnMessage);
}

EMSCRIPTEN_BINDINGS(state_machine_observer_callbacks) {
    class_<CallbackStateMachineObserver, base<StateMachineObserver>>("CallbackStateMachineObserver")
        .constructor<>()
        .function("setOnStart", &CallbackStateMachineObserver::setOnStart)
        .function("setOnStop", &CallbackStateMachineObserver::setOnStop)
        .function("setOnTransition", &CallbackStateMachineObserver::setOnTransition)
        .function("setOnStateEntered", &CallbackStateMachineObserver::setOnStateEntered)
        .function("setOnStateExit", &CallbackStateMachineObserver::setOnStateExit)
        .function("setOnCustomEvent", &CallbackStateMachineObserver::setOnCustomEvent)
        .function("setOnStringInputValueChange", &CallbackStateMachineObserver::setOnStringInputValueChange)
        .function("setOnNumericInputValueChange", &CallbackStateMachineObserver::setOnNumericInputValueChange)
        .function("setOnBooleanInputValueChange", &CallbackStateMachineObserver::setOnBooleanInputValueChange)
        .function("setOnInputFired", &CallbackStateMachineObserver::setOnInputFired)
        .function("setOnError", &CallbackStateMachineObserver::setOnError);
}

EMSCRIPTEN_BINDINGS(DotLottiePlayer)
{
    register_vector<float>("VectorFloat");
    register_vector<std::string>("VectorString");
    register_vector<Marker>("VectorMarker");
    
    register_optional<std::vector<float>>();
    register_optional<std::string>();
    register_optional<Layout>();
    register_optional<uint32_t>();
    register_optional<bool>();
    register_optional<Mode>();
    register_optional<float>();
    

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

    value_object<OpenUrlPolicy>("OpenUrlPolicy")
        .field("requireUserInteraction", &OpenUrlPolicy::require_user_interaction)
        .field("whitelist", &OpenUrlPolicy::whitelist);

    function("createDefaultOpenUrlPolicy", &create_default_open_url_policy);

    value_object<Marker>("Marker")
        .field("name", &Marker::name)
        .field("time", &Marker::time)
        .field("duration", &Marker::duration);

    value_object<Config>("Config")
        .field("autoplay", &Config::autoplay)
        .field("loopAnimation", &Config::loop_animation)
        .field("loopCount", &Config::loop_count)
        .field("mode", &Config::mode)
        .field("speed", &Config::speed)
        .field("useFrameInterpolation", &Config::use_frame_interpolation)
        .field("segment", &Config::segment)
        .field("backgroundColor", &Config::background_color)
        .field("layout", &Config::layout)
        .field("marker", &Config::marker)
        .field("themeId", &Config::theme_id)
        .field("stateMachineId", &Config::state_machine_id)
        .field("animationId", &Config::animation_id);

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
        .function("on_complete", &Observer::on_complete, pure_virtual());

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
        .function("on_error", &StateMachineObserver::on_error, pure_virtual());

    class_<StateMachineInternalObserver>("StateMachineInternalObserver")
        .smart_ptr<std::shared_ptr<StateMachineInternalObserver>>("StateMachineInternalObserver")
        .function("on_message", &StateMachineInternalObserver::on_message, pure_virtual());

    class_<DotLottiePlayer>("DotLottiePlayer")
        .smart_ptr<std::shared_ptr<DotLottiePlayer>>("DotLottiePlayer")
        .constructor(&DotLottiePlayer::init)
        .function("buffer", &buffer)
        .function("clear", &DotLottiePlayer::clear)
        .function("config", &DotLottiePlayer::config)
        .function("currentFrame", &DotLottiePlayer::current_frame)
        .function("duration", &DotLottiePlayer::duration)
        .function("isLoaded", &DotLottiePlayer::is_loaded)
        .function("isPaused", &DotLottiePlayer::is_paused)
        .function("isPlaying", &DotLottiePlayer::is_playing)
        .function("isStopped", &DotLottiePlayer::is_stopped)
        .function("loadAnimationData", &DotLottiePlayer::load_animation_data)
        .function("loadAnimationPath", &DotLottiePlayer::load_animation_path)
        .function("loadDotLottieData", &load_dotlottie_data)
        .function("loadAnimation", &DotLottiePlayer::load_animation)
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
        .function("unsubscribe", &unsubscribe)

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
        .function("stateMachineSubscribe", &stateMachineSubscribe, allow_raw_pointers())
        .function("stateMachineUnsubscribe", &stateMachineUnsubscribe)
        .function("stateMachineInternalSubscribe", &stateMachineInternalSubscribe, allow_raw_pointers())
        .function("stateMachineInternalUnsubscribe", &stateMachineInternalUnsubscribe)
        .function("stateMachineStatus", &DotLottiePlayer::state_machine_status);
}