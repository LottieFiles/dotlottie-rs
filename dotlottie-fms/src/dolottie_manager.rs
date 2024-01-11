use std::{collections::HashMap, ops::Index};

use crate::{get_manifest, Animation, DotLottieError, Manifest, ManifestAnimation};

pub struct DotLottieManager {
    current_animation_id: String,
    manifest: Manifest,
    zip_data: Vec<u8>,
    animation_settings_cache: HashMap<String, ManifestAnimation>,
}

impl DotLottieManager {
    pub fn new(dotlottie: Option<Vec<u8>>) -> Self {
        if let Some(dotlottie) = dotlottie {
            // Initialize the manager with the dotLottie file
            let manifest = get_manifest(&dotlottie).unwrap();
            let mut id = String::new();

            if let Some(first_animation) = &manifest.active_animation_id {
                id = first_animation.clone();
            } else if let Ok(animations) = manifest.animations.lock() {
                id = animations.index(0).id.clone();
            } else {
                panic!("No animations found in dotLottie file");
            }

            DotLottieManager {
                current_animation_id: id,
                manifest,
                zip_data: dotlottie,
                animation_settings_cache: HashMap::new(),
            }
        } else {
            DotLottieManager {
                current_animation_id: String::new(),
                manifest: Manifest::new(),
                zip_data: vec![],
                animation_settings_cache: HashMap::new(),
            }
        }

        // // Initialize the manager with the dotLottie file
        // let manifest = get_manifest(&dotlottie).unwrap();
        // let mut id = String::new();

        // if let Some(first_animation) = &manifest.active_animation_id {
        //     id = first_animation.clone();
        // } else if let Ok(animations) = manifest.animations.lock() {
        //     id = animations.index(0).id.clone();
        // } else {
        //     panic!("No animations found in dotLottie file");
        // }

        // DotLottieManager {
        //     current_animation_id: String::new(),
        //     manifest: Manifest::new(),
        //     zip_data: vec![],
        //     animation_settings_cache: HashMap::new(),
        // }
    }

    // pub fn new() -> Self {
    // Initialize the manager with the dotLottie file
    // let manifest = get_manifest(&dotlottie).unwrap();
    // let mut id = String::new();

    // if let Some(first_animation) = &manifest.active_animation_id {
    //     id = first_animation.clone();
    // } else if let Ok(animations) = manifest.animations.lock() {
    //     id = animations.index(0).id.clone();
    // } else {
    //     panic!("No animations found in dotLottie file");
    // }
    //     DotLottieManager {
    //         current_animation_id: String::new(),
    //         manifest: Manifest::new(),
    //         zip_data: vec![],
    //         animation_settings_cache: HashMap::new(),
    //     }
    // }

    pub fn init(dotlottie: Vec<u8>) -> Self {
        // Initialize the manager with the dotLottie file
        let manifest = get_manifest(&dotlottie).unwrap();
        let mut id = String::new();

        if let Some(first_animation) = &manifest.active_animation_id {
            id = first_animation.clone();
        } else if let Ok(animations) = manifest.animations.lock() {
            id = animations.index(0).id.clone();
        } else {
            panic!("No animations found in dotLottie file");
        }

        DotLottieManager {
            current_animation_id: id,
            manifest,
            zip_data: dotlottie,
            animation_settings_cache: HashMap::new(),
        }
    }

    /// Advances to the next animation and returns it's animation data as a string.
    pub fn next_animation(&mut self) -> Result<String, DotLottieError> {
        let mut i = 0;

        if let Ok(animations) = self.manifest.animations.lock() {
            for anim in animations.iter() {
                if anim.id == self.current_animation_id {
                    // return Result::Ok(true);
                    if i + 1 < animations.len() {
                        self.current_animation_id = animations[i + 1].id.clone();
                    }
                }
                i += 1;
            }
        }

        self.get_animation(&self.current_animation_id)
    }

    /// Reverses to the previous animation and returns it's animation data as a string.
    pub fn previous_animation(&mut self) -> Result<String, DotLottieError> {
        if let Ok(animations) = self.manifest.animations.lock() {
            let mut i = animations.len();

            for anim in animations.iter() {
                if anim.id == self.current_animation_id {
                    if i - 1 > 0 {
                        self.current_animation_id = animations[i + 1].id.clone();
                    }
                }
                i -= 1;
            }
        }

        self.get_animation(&self.current_animation_id)
    }

    /// Returns the playback settings for the animation with the given ID.
    /// Memoizes the settings in a HashMap for faster access.
    pub fn get_playback_settings(
        &mut self,
        animation_id: &str,
    ) -> Result<ManifestAnimation, DotLottieError> {
        if let Some(manifest_animation) = self.animation_settings_cache.get(animation_id) {
            let cloned_animation = manifest_animation.clone(); // Clone the value

            return Result::Ok(cloned_animation);
        }

        if let Ok(animations) = self.manifest.animations.lock() {
            for anim in animations.iter() {
                if &anim.id == animation_id {
                    self.animation_settings_cache
                        .insert(animation_id.to_string().clone(), anim.clone());

                    return Result::Ok(anim.clone());
                }
            }
        }

        return Result::Err(DotLottieError::MutexLockError);
    }

    pub fn contains_animation(&self, animation_id: &str) -> Result<bool, DotLottieError> {
        if let Ok(animations) = self.manifest.animations.lock() {
            for anim in animations.iter() {
                if anim.id == animation_id {
                    return Result::Ok(true);
                }
            }

            return Result::Ok(false);
        }

        return Result::Err(DotLottieError::MutexLockError);
    }

    pub fn current_animation_playback_settings(
        &mut self,
    ) -> Result<ManifestAnimation, DotLottieError> {
        let animation_id = self.current_animation_id.clone();

        self.get_playback_settings(&animation_id)
    }

    pub fn get_animation(&self, animation_id: &str) -> Result<String, DotLottieError> {
        crate::get_animation(&self.zip_data, animation_id)
    }

    pub fn get_animations(&self) -> Result<Vec<Animation>, DotLottieError> {
        crate::get_animations(&self.zip_data)
    }

    pub fn set_active_animation(&mut self, animation_id: &str) -> Result<String, DotLottieError> {
        if let Ok(contains) = self.contains_animation(animation_id) {
            if contains {
                self.current_animation_id = animation_id.to_string();

                return Result::Ok(self.get_animation(animation_id)?);
            }
        }

        return Result::Err(DotLottieError::AnimationNotFound {
            animation_id: animation_id.to_string(),
        });
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn current_animation_id(&self) -> &str {
        &self.current_animation_id.as_str()
    }
}
