use std::{collections::HashMap, ops::Index};

use crate::{get_manifest, AnimationContainer, DotLottieError, Manifest, ManifestAnimation};

pub struct DotLottieManager {
    active_animation_id: String,
    manifest: Manifest,
    zip_data: Vec<u8>,
    animation_settings_cache: HashMap<String, ManifestAnimation>,
    animation_data_cache: HashMap<String, String>,
    theme_cache: HashMap<String, String>,
}

impl DotLottieManager {
    pub fn new(dotlottie: Option<Vec<u8>>) -> Result<Self, DotLottieError> {
        if let Some(dotlottie) = dotlottie {
            // Initialize the manager with the dotLottie file
            let manifest = get_manifest(&dotlottie);

            match manifest {
                Ok(manifest) => {
                    let id: String;

                    if let Some(first_animation) = &manifest.active_animation_id {
                        id = first_animation.clone();
                    } else if manifest.animations.len() > 0 {
                        id = manifest.animations.index(0).id.clone();
                    } else {
                        return Err(DotLottieError::AnimationsNotFound);
                    }

                    Ok(DotLottieManager {
                        active_animation_id: id,
                        manifest,
                        zip_data: dotlottie,
                        animation_settings_cache: HashMap::new(),
                        animation_data_cache: HashMap::new(),
                        theme_cache: HashMap::new(),
                    })
                }
                Err(error) => Err(error),
            }
        } else {
            Ok(DotLottieManager {
                active_animation_id: String::new(),
                manifest: Manifest::new(),
                zip_data: vec![],
                animation_settings_cache: HashMap::new(),
                animation_data_cache: HashMap::new(),
                theme_cache: HashMap::new(),
            })
        }
    }

    pub fn init(&mut self, dotlottie: &[u8]) -> Result<bool, DotLottieError> {
        // Initialize the manager with the dotLottie file
        let manifest = get_manifest(dotlottie);

        match manifest {
            Ok(manifest) => {
                let id: String;

                if let Some(first_animation) = &manifest.active_animation_id {
                    id = first_animation.clone();
                } else if manifest.animations.len() > 0 {
                    id = manifest.animations.index(0).id.clone();
                } else {
                    return Err(DotLottieError::AnimationsNotFound);
                }

                self.active_animation_id = id;
                self.manifest = manifest;
                self.zip_data = dotlottie.to_vec();

                return Ok(true);
            }
            Err(error) => Err(error),
        }
    }

    /// Advances to the next animation and returns it's animation data as a string.
    #[allow(dead_code)]
    fn next_animation(&mut self) -> Result<String, DotLottieError> {
        let mut i = 0;
        let new_active_animation_id: String;

        for anim in self.manifest.animations.iter() {
            if anim.id == self.active_animation_id {
                if i + 1 < self.manifest.animations.len() {
                    self.active_animation_id = self.manifest.animations[i + 1].id.clone();

                    new_active_animation_id = self.manifest.animations[i + 1].id.clone();

                    return self.get_animation(&new_active_animation_id);
                }
            }
            i += 1;
        }

        // std::mem::drop(animations);

        let active_animation_id = self.active_animation_id.clone();

        return self.get_animation(&active_animation_id);
    }

    /// Reverses to the previous animation and returns it's animation data as a string.
    #[allow(dead_code)]
    fn previous_animation(&mut self) -> Result<String, DotLottieError> {
        let new_active_animation_id: String;
        let mut i = 0;

        for anim in self.manifest.animations.iter() {
            if anim.id == self.active_animation_id {
                if i > 0 {
                    self.active_animation_id = self.manifest.animations[i - 1].id.clone();

                    new_active_animation_id = self.manifest.animations[i - 1].id.clone();

                    return self.get_animation(&new_active_animation_id);
                }
            }
            i += 1;
        }

        let active_animation_id = self.active_animation_id.clone();

        self.get_animation(&active_animation_id)
    }

    /// Returns the playback settings for the animation with the given ID.
    /// Memoizes the settings in a HashMap for faster access.
    pub fn get_playback_settings(
        &mut self,
        animation_id: &str,
    ) -> Result<ManifestAnimation, DotLottieError> {
        if let Some(manifest_animation) = self.animation_settings_cache.get(animation_id) {
            let cloned_animation = manifest_animation.clone();

            return Ok(cloned_animation);
        }

        if self.manifest.animations.len() > 0 {
            for anim in self.manifest.animations.iter() {
                if &anim.id == animation_id {
                    self.animation_settings_cache
                        .insert(animation_id.to_string().clone(), anim.clone());

                    return Ok(anim.clone());
                }
            }
        }

        return Err(DotLottieError::AnimationNotFound {
            animation_id: animation_id.to_string(),
        });
    }

    pub fn contains_animation(&self, animation_id: &str) -> Result<bool, DotLottieError> {
        if self.manifest.animations.len() > 0 {
            for anim in self.manifest.animations.iter() {
                if anim.id == animation_id {
                    return Ok(true);
                }
            }

            return Ok(false);
        }

        return Err(DotLottieError::MutexLockError);
    }

    pub fn get_active_animation(&mut self) -> Result<String, DotLottieError> {
        let active_animation_id = self.active_animation_id.clone();

        self.get_animation(&active_animation_id)
    }

    pub fn active_animation_playback_settings(
        &mut self,
    ) -> Result<ManifestAnimation, DotLottieError> {
        let animation_id = self.active_animation_id.clone();

        self.get_playback_settings(&animation_id)
    }

    /// Returns the animation data for the animation with the given ID.
    /// Memoizes the animation data in a HashMap for faster access.
    pub fn get_animation(&mut self, animation_id: &str) -> Result<String, DotLottieError> {
        if let Some(animation) = self.animation_data_cache.get(animation_id) {
            let cloned_animation = animation.clone(); // Clone the value

            return Ok(cloned_animation);
        } else {
            let animation = crate::get_animation(&self.zip_data, animation_id);

            if let Ok(animation) = animation {
                self.animation_data_cache
                    .insert(animation_id.to_string().clone(), animation.clone());

                return Ok(animation);
            } else {
                return Err(DotLottieError::AnimationNotFound {
                    animation_id: animation_id.to_string(),
                });
            }
        }
    }

    pub fn get_animations(&self) -> Result<Vec<AnimationContainer>, DotLottieError> {
        crate::get_animations(&self.zip_data)
    }

    pub fn set_active_animation(&mut self, animation_id: &str) -> Result<String, DotLottieError> {
        if let Ok(contains) = self.contains_animation(animation_id) {
            if contains {
                self.active_animation_id = animation_id.to_string();

                return Ok(self.get_animation(animation_id)?);
            }
        }

        return Err(DotLottieError::AnimationNotFound {
            animation_id: animation_id.to_string(),
        });
    }

    pub fn manifest(&self) -> Option<Manifest> {
        if self.manifest.animations.len() == 0 {
            return None;
        }

        let mut manifest = Manifest::new();

        manifest.active_animation_id = Some(self.active_animation_id.clone());
        manifest.animations = self.manifest.animations.clone().into();
        manifest.author = self.manifest.author.clone();
        manifest.description = self.manifest.description.clone();
        manifest.generator = self.manifest.generator.clone();
        manifest.keywords = self.manifest.keywords.clone();
        manifest.revision = self.manifest.revision;
        manifest.themes = self.manifest.themes.clone();
        manifest.states = self.manifest.states.clone();
        manifest.version = self.manifest.version.clone();

        Some(manifest)
    }

    pub fn active_animation_id(&self) -> String {
        self.active_animation_id.clone()
    }

    pub fn get_theme(&mut self, theme_id: &str) -> Result<String, DotLottieError> {
        if let Some(theme) = self.theme_cache.get(theme_id) {
            return Ok(theme.clone());
        }

        let theme = crate::get_theme(&self.zip_data, theme_id)?;

        self.theme_cache.insert(theme_id.to_string(), theme.clone());

        Ok(theme)
    }
}
