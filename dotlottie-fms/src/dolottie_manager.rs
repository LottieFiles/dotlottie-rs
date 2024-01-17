use std::{collections::HashMap, ops::Index};

use crate::{get_manifest, Animation, DotLottieError, Manifest, ManifestAnimation};

pub struct DotLottieManager {
    current_animation_id: String,
    manifest: Manifest,
    zip_data: Vec<u8>,
    animation_settings_cache: HashMap<String, ManifestAnimation>,
    animation_data_cache: HashMap<String, String>,
}

impl DotLottieManager {
    pub fn new(dotlottie: Option<Vec<u8>>) -> Self {
        if let Some(dotlottie) = dotlottie {
            // Initialize the manager with the dotLottie file
            let manifest = get_manifest(&dotlottie);

            match manifest {
                Ok(manifest) => {
                    let mut id = String::new();

                    if let Some(first_animation) = &manifest.active_animation_id {
                        id = first_animation.clone();
                    } else if let Ok(animations) = manifest.animations.read() {
                        id = animations.index(0).id.clone();
                    } else {
                        panic!("No animations found in dotLottie file");
                    }

                    DotLottieManager {
                        current_animation_id: id,
                        manifest,
                        zip_data: dotlottie,
                        animation_settings_cache: HashMap::new(),
                        animation_data_cache: HashMap::new(),
                    }
                }
                Err(error) => {
                    eprintln!("Unable to initialize dotLottie manager: {}", error);

                    DotLottieManager {
                        current_animation_id: String::new(),
                        manifest: Manifest::new(),
                        zip_data: vec![],
                        animation_settings_cache: HashMap::new(),
                        animation_data_cache: HashMap::new(),
                    }
                }
            }
        } else {
            DotLottieManager {
                current_animation_id: String::new(),
                manifest: Manifest::new(),
                zip_data: vec![],
                animation_settings_cache: HashMap::new(),
                animation_data_cache: HashMap::new(),
            }
        }
    }

    pub fn init(&mut self, dotlottie: Vec<u8>) {
        // Initialize the manager with the dotLottie file
        let manifest = get_manifest(&dotlottie);

        match manifest {
            Ok(manifest) => {
                let mut id = String::new();

                if let Some(first_animation) = &manifest.active_animation_id {
                    id = first_animation.clone();
                } else if let Ok(animations) = manifest.animations.read() {
                    id = animations.index(0).id.clone();
                } else {
                    panic!("No animations found in dotLottie file");
                }

                self.current_animation_id = id;
                self.manifest = manifest;
                self.zip_data = dotlottie;
            }
            Err(error) => {
                eprintln!("Unable to initialize dotLottie manager: {}", error);
            }
        }
    }

    /// Advances to the next animation and returns it's animation data as a string.
    fn next_animation(&mut self) -> Result<String, DotLottieError> {
        let mut i = 0;
        let mut new_current_animation_id = self.current_animation_id.clone();
        let animations = match self.manifest.animations.read() {
            Ok(animations) => animations,
            Err(_) => return Err(DotLottieError::MutexLockError),
        };

        for anim in animations.iter() {
            if anim.id == self.current_animation_id {
                if i + 1 < animations.len() {
                    self.current_animation_id = animations[i + 1].id.clone();

                    new_current_animation_id = animations[i + 1].id.clone();

                    std::mem::drop(animations);

                    return self.get_animation(&new_current_animation_id);
                }
            }
            i += 1;
        }

        std::mem::drop(animations);

        let current_animation_id = self.current_animation_id.clone();

        return self.get_animation(&current_animation_id);
    }

    /// Reverses to the previous animation and returns it's animation data as a string.
    fn previous_animation(&mut self) -> Result<String, DotLottieError> {
        let mut new_current_animation_id = self.current_animation_id.clone();
        let animations = match self.manifest.animations.read() {
            Ok(animations) => animations,
            Err(_) => return Err(DotLottieError::MutexLockError),
        };
        let mut i = 0;

        for anim in animations.iter() {
            if anim.id == self.current_animation_id {
                if i > 0 {
                    self.current_animation_id = animations[i - 1].id.clone();

                    new_current_animation_id = animations[i - 1].id.clone();
                    std::mem::drop(animations);

                    return self.get_animation(&new_current_animation_id);
                }
            }
            i += 1;
        }

        std::mem::drop(animations);

        let current_animation_id = self.current_animation_id.clone();

        self.get_animation(&current_animation_id)
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

        if let Ok(animations) = self.manifest.animations.read() {
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
        if let Ok(animations) = self.manifest.animations.read() {
            for anim in animations.iter() {
                if anim.id == animation_id {
                    return Result::Ok(true);
                }
            }

            return Result::Ok(false);
        }

        return Result::Err(DotLottieError::MutexLockError);
    }

    pub fn get_current_animation(&mut self) -> Result<String, DotLottieError> {
        let current_animation_id = self.current_animation_id.clone();

        self.get_animation(&current_animation_id)
    }

    pub fn current_animation_playback_settings(
        &mut self,
    ) -> Result<ManifestAnimation, DotLottieError> {
        let animation_id = self.current_animation_id.clone();

        self.get_playback_settings(&animation_id)
    }

    /// Returns the animation data for the animation with the given ID.
    /// Memoizes the animation data in a HashMap for faster access.
    pub fn get_animation(&mut self, animation_id: &str) -> Result<String, DotLottieError> {
        if let Some(animation) = self.animation_data_cache.get(animation_id) {
            let cloned_animation = animation.clone(); // Clone the value

            return Result::Ok(cloned_animation);
        } else {
            let animation = crate::get_animation(&self.zip_data, animation_id);

            if let Ok(animation) = animation {
                self.animation_data_cache
                    .insert(animation_id.to_string().clone(), animation.clone());

                return Result::Ok(animation);
            } else {
                return Result::Err(DotLottieError::AnimationNotFound {
                    animation_id: animation_id.to_string(),
                });
            }
        }
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

    pub fn manifest(&self) -> Manifest {
        let mut manifest = Manifest::new();

        manifest.active_animation_id = Some(self.current_animation_id.clone());
        manifest.animations = self.manifest.animations.read().unwrap().clone().into();
        manifest.author = self.manifest.author.clone();
        manifest.description = self.manifest.description.clone();
        manifest.generator = self.manifest.generator.clone();
        manifest.keywords = self.manifest.keywords.clone();
        manifest.revision = self.manifest.revision;
        manifest.themes = self.manifest.themes.clone();
        manifest.states = self.manifest.states.clone();
        manifest.version = self.manifest.version.clone();

        manifest
    }

    pub fn current_animation_id(&self) -> String {
        self.current_animation_id.clone()
    }
}
