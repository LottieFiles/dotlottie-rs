use std::sync::Arc;

pub struct ResolvedAsset {
    pub data: Vec<u8>,
    pub mimetype: String,
}

pub trait AssetResolver: Send + Sync {
    fn resolve(&self, src: &str) -> Option<ResolvedAsset>;
}

pub struct AssetResolverContext {
    user_resolver: Option<Arc<dyn AssetResolver>>,
    #[cfg(feature = "dotlottie")]
    dotlottie_resolver: Option<crate::dotlottie::DotLottieAssetResolver>,
}

impl AssetResolverContext {
    pub(crate) fn new(
        user_resolver: Option<Arc<dyn AssetResolver>>,
        #[cfg(feature = "dotlottie")] dotlottie_resolver: Option<
            crate::dotlottie::DotLottieAssetResolver,
        >,
    ) -> Self {
        Self {
            user_resolver,
            #[cfg(feature = "dotlottie")]
            dotlottie_resolver,
        }
    }

    pub(crate) fn resolve(&self, src: &str) -> Option<ResolvedAsset> {
        if let Some(ref resolver) = self.user_resolver {
            if let Some(asset) = resolver.resolve(src) {
                return Some(asset);
            }
        }

        #[cfg(feature = "dotlottie")]
        if let Some(ref resolver) = self.dotlottie_resolver {
            if let Some(asset) = resolver.resolve(src) {
                return Some(asset);
            }
        }

        None
    }
}
