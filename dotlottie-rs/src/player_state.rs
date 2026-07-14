use crate::tween::TweenState;

pub(crate) enum State {
    Idle,
    Stopped,
    Paused,
    Playing,
    Tweening { tween: TweenState, resume: Resume },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Resume {
    Stopped,
    Paused,
    Playing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TweenOutcome {
    Completed,
    Cancelled,
}

impl From<Resume> for State {
    fn from(resume: Resume) -> Self {
        match resume {
            Resume::Stopped => State::Stopped,
            Resume::Paused => State::Paused,
            Resume::Playing => State::Playing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resume_maps_to_matching_state() {
        assert!(matches!(State::from(Resume::Stopped), State::Stopped));
        assert!(matches!(State::from(Resume::Paused), State::Paused));
        assert!(matches!(State::from(Resume::Playing), State::Playing));
    }
}
