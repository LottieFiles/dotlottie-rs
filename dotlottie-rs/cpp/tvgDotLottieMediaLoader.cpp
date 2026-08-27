#include "tvgCommon.h"
#include "tvgMediaLoader.h"

extern "C" {
    void* dlMediaOpen(const uint8_t* data, uint32_t size);
    void dlMediaClose(void* player);
    int dlMediaSync(void* player, const uint8_t** frame, uint32_t* w, uint32_t* h, float* duration, float* time);
    void dlMediaSeek(void* player, float seconds);
    void dlMediaSetPlaying(void* player, int on);
    void dlMediaSetVolume(void* player, float volume);
    void dlMediaSetMute(void* player, int on);
}

struct DotLottieMediaLoader : MediaLoader
{
    void* player = nullptr;

    ~DotLottieMediaLoader() override
    {
        if (player) dlMediaClose(player);
        tvg::free(surface.buf32);
    }

    bool open(const char* data, uint32_t size, const LoaderOps* ops, bool copy) override
    {
        player = dlMediaOpen(reinterpret_cast<const uint8_t*>(data), size);
        return player != nullptr;
    }

    bool sync() override
    {
        if (!player) return false;

        const uint8_t* frame = nullptr;
        uint32_t width = 0, height = 0;
        float duration = 0.0f, time = 0.0f;
        auto fresh = dlMediaSync(player, &frame, &width, &height, &duration, &time);

        if (width == 0 || height == 0) return false;

        totalTime = duration;
        curTime = time;

        if (width != surface.w || height != surface.h) {
            auto size = width * height * sizeof(uint32_t);
            surface.setup(tvg::realloc<uint32_t>(surface.buf32, size), width, width, height, sizeof(uint32_t), ColorSpace::ABGR8888S);
            if (surface.buf32) memset(surface.buf32, 0, size);
            w = static_cast<float>(width);
            h = static_cast<float>(height);
        }

        if (!fresh || !frame || !surface.buf32) return false;

        memcpy(surface.buf32, frame, surface.stride * surface.h * surface.channelSize);
        surface.premultiplied = false;
        return true;
    }

    RenderSurface* bitmap() override
    {
        sync();
        return BitmapLoader::bitmap();
    }

    Result play() override
    {
        paused = false;
        dlMediaSetPlaying(player, 1);
        return Result::Success;
    }

    Result pause() override
    {
        paused = true;
        dlMediaSetPlaying(player, 0);
        return Result::Success;
    }

    Result stop() override
    {
        paused = true;
        curTime = 0.0f;
        dlMediaSetPlaying(player, 0);
        dlMediaSeek(player, 0.0f);
        return Result::Success;
    }

    Result seek(float seconds) override
    {
        curTime = seconds;
        dlMediaSeek(player, seconds);
        return Result::Success;
    }

    Result loop(bool on) override
    {
        looping = on;
        return Result::Success;
    }

    Result volume(float volume) override
    {
        audioVolume = volume;
        dlMediaSetVolume(player, volume);
        return Result::Success;
    }

    Result mute(bool on) override
    {
        muted = on;
        dlMediaSetMute(player, on ? 1 : 0);
        return Result::Success;
    }
};

MediaLoader* MediaLoader::gen()
{
    return new DotLottieMediaLoader;
}
