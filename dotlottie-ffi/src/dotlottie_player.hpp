#pragma once

#include <bit>
#include <cstdint>
#include <exception>
#include <functional>
#include <iostream>
#include <map>
#include <memory>
#include <mutex>
#include <optional>
#include <stdexcept>
#include <streambuf>
#include <type_traits>
#include <variant>

#include "dotlottie_player_scaffolding.hpp"

namespace dotlottie_player {
struct DotLottiePlayer; 
struct Config; 
struct Layout; 
struct Marker;
enum class Fit;
enum class Mode;


enum class Fit: int32_t {
    kContain = 1,
    kFill = 2,
    kCover = 3,
    kFitWidth = 4,
    kFitHeight = 5,
    kNone = 6
};


enum class Mode: int32_t {
    kForward = 1,
    kReverse = 2,
    kBounce = 3,
    kReverseBounce = 4
};


struct Layout {
    Fit fit;
    std::vector<float> align;
};


struct Config {
    bool autoplay;
    bool loop_animation;
    Mode mode;
    float speed;
    bool use_frame_interpolation;
    std::vector<float> segment;
    uint32_t background_color;
    Layout layout;
    std::string marker;
};

namespace uniffi {
    struct FfiConverterDotLottiePlayer;
} // namespace uniffi

struct DotLottiePlayer {
    friend uniffi::FfiConverterDotLottiePlayer;

    DotLottiePlayer() = delete;

    DotLottiePlayer(const DotLottiePlayer &) = delete;
    DotLottiePlayer(DotLottiePlayer &&) = delete;

    DotLottiePlayer &operator=(const DotLottiePlayer &) = delete;
    DotLottiePlayer &operator=(DotLottiePlayer &&) = delete;

    ~DotLottiePlayer();
    static std::shared_ptr<DotLottiePlayer> init(const Config &config);
    std::string active_animation_id();
    std::string active_theme_id();
    std::vector<float> animation_size();
    uint64_t buffer_len();
    uint64_t buffer_ptr();
    void clear();
    Config config();
    float current_frame();
    float duration();
    bool is_complete();
    bool is_loaded();
    bool is_paused();
    bool is_playing();
    bool is_stopped();
    bool load_animation(const std::string &animation_id, uint32_t width, uint32_t height);
    bool load_animation_data(const std::string &animation_data, uint32_t width, uint32_t height);
    bool load_animation_path(const std::string &animation_path, uint32_t width, uint32_t height);
    bool load_dotlottie_data(const std::vector<uint8_t> &file_data, uint32_t width, uint32_t height);
    bool load_state_machine(const std::string &str);
    bool load_state_machine_data(const std::string &state_machine);
    bool load_theme(const std::string &theme_id);
    bool load_theme_data(const std::string &theme_data);
    uint32_t loop_count();
    std::string manifest_string();
    std::vector<Marker> markers();
    bool pause();
    bool play();
    int32_t post_serialized_event(const std::string &event);
    bool render();
    float request_frame();
    bool resize(uint32_t width, uint32_t height);
    bool seek(float no);
    float segment_duration();
    void set_config(const Config &config);
    bool set_frame(float no);
    bool set_state_machine_boolean_context(const std::string &key, bool value);
    bool set_state_machine_numeric_context(const std::string &key, float value);
    bool set_state_machine_string_context(const std::string &key, const std::string &value);
    bool set_viewport(int32_t x, int32_t y, int32_t w, int32_t h);
    bool start_state_machine();
    std::vector<std::string> state_machine_framework_setup();
    bool stop();
    bool stop_state_machine();
    float total_frames();

private:
    DotLottiePlayer(void *);

    void *instance;
};


struct Marker {
    std::string name;
    float time;
    float duration;
};

namespace uniffi {struct RustStreamBuffer: std::basic_streambuf<char> {
    RustStreamBuffer(RustBuffer *buf) {
        char* data = reinterpret_cast<char*>(buf->data);
        this->setg(data, data, data + buf->len);
        this->setp(data, data + buf->capacity);
    }
    ~RustStreamBuffer() = default;

private:
    RustStreamBuffer() = delete;
    RustStreamBuffer(const RustStreamBuffer &) = delete;
    RustStreamBuffer(RustStreamBuffer &&) = delete;

    RustStreamBuffer &operator=(const RustStreamBuffer &) = delete;
    RustStreamBuffer &operator=(RustStreamBuffer &&) = delete;
};

struct RustStream: std::basic_iostream<char> {
    RustStream(RustBuffer *buf):
        std::basic_iostream<char>(&streambuf), streambuf(RustStreamBuffer(buf)) { }

    template <typename T, typename = std::enable_if_t<std::is_arithmetic_v<T>>>
    RustStream &operator>>(T &val) {
        read(reinterpret_cast<char *>(&val), sizeof(T));

        if (std::endian::native != std::endian::big) {
            auto bytes = reinterpret_cast<char *>(&val);

            std::reverse(bytes, bytes + sizeof(T));
        }

        return *this;
    }

    template <typename T, typename = std::enable_if_t<std::is_arithmetic_v<T>>>
    RustStream &operator<<(T val) {
        if (std::endian::native != std::endian::big) {
            auto bytes = reinterpret_cast<char *>(&val);

            std::reverse(bytes, bytes + sizeof(T));
        }

        write(reinterpret_cast<char *>(&val), sizeof(T));

        return *this;
    }
private:
    RustStreamBuffer streambuf;
};


RustBuffer rustbuffer_alloc(int32_t);
RustBuffer rustbuffer_from_bytes(const ForeignBytes &);
void rustbuffer_free(RustBuffer);

struct FfiConverterUInt32 {
    static uint32_t lift(uint32_t);
    static uint32_t lower(uint32_t);
    static uint32_t read(RustStream &);
    static void write(RustStream &, uint32_t);
    static int32_t allocation_size(uint32_t);
};

struct FfiConverterInt32 {
    static int32_t lift(int32_t);
    static int32_t lower(int32_t);
    static int32_t read(RustStream &);
    static void write(RustStream &, int32_t);
    static int32_t allocation_size(int32_t);
};

struct FfiConverterUInt64 {
    static uint64_t lift(uint64_t);
    static uint64_t lower(uint64_t);
    static uint64_t read(RustStream &);
    static void write(RustStream &, uint64_t);
    static int32_t allocation_size(uint64_t);
};

struct FfiConverterFloat {
    static float lift(float);
    static float lower(float);
    static float read(RustStream &);
    static void write(RustStream &, float);
    static int32_t allocation_size(float);
};

struct FfiConverterBool {
    static bool lift(uint8_t);
    static uint8_t lower(bool);
    static bool read(RustStream &);
    static void write(RustStream &, bool);
    static int32_t allocation_size(bool);
};
struct FfiConverterString {
    static std::string lift(RustBuffer buf);
    static RustBuffer lower(const std::string &);
    static std::string read(RustStream &);
    static void write(RustStream &, const std::string &);
    static int32_t allocation_size(const std::string &);
};

struct FfiConverterBytes {
    static std::vector<uint8_t> lift(RustBuffer);
    static RustBuffer lower(const std::vector<uint8_t> &);
    static std::vector<uint8_t> read(RustStream &);
    static void write(RustStream &, const std::vector<uint8_t> &);
    static int32_t allocation_size(const std::vector<uint8_t> &);
};

struct FfiConverterDotLottiePlayer {
    static std::shared_ptr<DotLottiePlayer> lift(void *);
    static void *lower(const std::shared_ptr<DotLottiePlayer> &);
    static std::shared_ptr<DotLottiePlayer> read(RustStream &);
    static void write(RustStream &, const std::shared_ptr<DotLottiePlayer> &);
    static int32_t allocation_size(const std::shared_ptr<DotLottiePlayer> &);
};

struct FfiConverterTypeConfig {
    static Config lift(RustBuffer);
    static RustBuffer lower(const Config &);
    static Config read(RustStream &);
    static void write(RustStream &, const Config &);
    static int32_t allocation_size(const Config &);
};

struct FfiConverterTypeLayout {
    static Layout lift(RustBuffer);
    static RustBuffer lower(const Layout &);
    static Layout read(RustStream &);
    static void write(RustStream &, const Layout &);
    static int32_t allocation_size(const Layout &);
};

struct FfiConverterTypeMarker {
    static Marker lift(RustBuffer);
    static RustBuffer lower(const Marker &);
    static Marker read(RustStream &);
    static void write(RustStream &, const Marker &);
    static int32_t allocation_size(const Marker &);
};

struct FfiConverterTypeFit {
    static Fit lift(RustBuffer);
    static RustBuffer lower(const Fit &);
    static Fit read(RustStream &);
    static void write(RustStream &, const Fit &);
    static int32_t allocation_size(const Fit &);
};

struct FfiConverterTypeMode {
    static Mode lift(RustBuffer);
    static RustBuffer lower(const Mode &);
    static Mode read(RustStream &);
    static void write(RustStream &, const Mode &);
    static int32_t allocation_size(const Mode &);
};

struct FfiConverterSequenceFloat {
    static std::vector<float> lift(RustBuffer);
    static RustBuffer lower(const std::vector<float> &);
    static std::vector<float> read(RustStream &);
    static void write(RustStream &, const std::vector<float> &);
    static int32_t allocation_size(const std::vector<float> &);
};

struct FfiConverterSequenceString {
    static std::vector<std::string> lift(RustBuffer);
    static RustBuffer lower(const std::vector<std::string> &);
    static std::vector<std::string> read(RustStream &);
    static void write(RustStream &, const std::vector<std::string> &);
    static int32_t allocation_size(const std::vector<std::string> &);
};

struct FfiConverterSequenceTypeMarker {
    static std::vector<Marker> lift(RustBuffer);
    static RustBuffer lower(const std::vector<Marker> &);
    static std::vector<Marker> read(RustStream &);
    static void write(RustStream &, const std::vector<Marker> &);
    static int32_t allocation_size(const std::vector<Marker> &);
};
} // namespace uniffi

Config create_default_config();
Layout create_default_layout();
} // namespace dotlottie_player