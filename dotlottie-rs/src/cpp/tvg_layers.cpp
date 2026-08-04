/*
 * Layer-name enumeration over ThorVG's internal Lottie model. The scene graph
 * only carries one-way djb2 hashes, so authored names must come from the
 * loader's model. Delete once upstream exposes accessor entities for Lottie.
 */

#include "tvgCommon.h"
#include "tvgPicture.h"
#include "tvgLottieLoader.h"
#include "tvgLottieModel.h"
#include "thorvg_capi.h"

using namespace tvg;

namespace {

int32_t emitLayers(LottieComposition* comp, LottieGroup* group, uint32_t depth,
                   void (*cb)(void*, const char*, uint32_t), void* ctx)
{
    if (depth > 8) return 0;  // guard against self-referencing precomp assets

    int32_t count = 0;
    ARRAY_FOREACH(p, group->children) {
        if ((*p)->type != LottieObject::Type::Layer) continue;
        auto layer = static_cast<LottieLayer*>(*p);
        if (layer->name) {
            cb(ctx, layer->name, depth);
            ++count;
        }
        if (layer->type == LottieLayer::Precomp && layer->rid) {
            if (auto asset = comp->asset(layer->rid)) {
                count += emitLayers(comp, asset, depth + 1, cb, ctx);
            }
        }
    }
    return count;
}

}

extern "C" int32_t dotlottie_layer_names(Tvg_Animation animation,
                                         void (*cb)(void* ctx, const char* name, uint32_t depth),
                                         void* ctx)
{
    if (!animation || !cb) return -1;

    auto picture = reinterpret_cast<Animation*>(animation)->picture();
    if (!picture) return -1;

    auto loader = static_cast<PictureImpl*>(picture)->loader;
    if (!loader || loader->type != FileType::Lot) return -1;

    auto lottie = static_cast<LottieLoader*>(loader);
    lottie->done();  // the model is produced by the async parse task

    if (!lottie->comp || !lottie->comp->root) return -1;
    return emitLayers(lottie->comp, lottie->comp->root, 0, cb, ctx);
}
