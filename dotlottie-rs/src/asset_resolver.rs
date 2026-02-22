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
    fms_resolver: Option<crate::fms::FmsAssetResolver>,
}

impl AssetResolverContext {
    pub(crate) fn new(
        user_resolver: Option<Arc<dyn AssetResolver>>,
        #[cfg(feature = "dotlottie")] fms_resolver: Option<crate::fms::FmsAssetResolver>,
    ) -> Self {
        Self {
            user_resolver,
            #[cfg(feature = "dotlottie")]
            fms_resolver,
        }
    }

    pub(crate) fn resolve(&self, src: &str) -> Option<ResolvedAsset> {
        if let Some(ref resolver) = self.user_resolver {
            if let Some(asset) = resolver.resolve(src) {
                return Some(asset);
            }
        }

        #[cfg(feature = "dotlottie")]
        if let Some(ref resolver) = self.fms_resolver {
            if let Some(asset) = resolver.resolve(src) {
                return Some(asset);
            }
        }

        None
    }
}
