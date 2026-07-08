use rodio::Player;

use crate::{
    audio::AudioAsset,
    core::{Base, Component, EngineApi, Handler},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackMode {
    Normal,
    Loop,
    OneShot,
}

pub struct Sound {
    sound: Handler<AudioAsset>,
    pub auto_play: bool,
    pub volume: f32,
    playing: Option<Player>,
    pub mode: PlaybackMode,
}

impl Sound {
    pub fn new(sound: Handler<AudioAsset>, mode: PlaybackMode) -> Self {
        Self {
            sound,
            auto_play: false,
            volume: 1.0,
            playing: None,
            mode,
        }
    }
    pub fn play(&mut self, ctx: &mut impl EngineApi) {
        match self.mode {
            PlaybackMode::OneShot => {
                let handle = ctx.play(self.sound, false);
                handle.set_volume(self.volume);
                handle.detach();
                self.playing = None;
            }
            PlaybackMode::Normal => {
                self.stop();
                let handle = ctx.play(self.sound, false);
                handle.set_volume(self.volume);
                self.playing = Some(handle);
            }
            PlaybackMode::Loop => {
                self.stop();
                let handle = ctx.play(self.sound, true);
                handle.set_volume(self.volume);
                self.playing = Some(handle);
            }
        }
    }
    pub fn stop(&mut self) {
        if let Some(handle) = &self.playing {
            handle.stop();
        }
        self.playing = None
    }
    pub fn pause(&mut self) {
        if let Some(handle) = &self.playing {
            handle.pause();
        }
    }
    pub fn resume(&mut self) {
        if let Some(handle) = &self.playing {
            handle.play();
        }
    }
    pub fn set_volume(&mut self, volume: f32) {
        if let Some(handle) = &self.playing {
            handle.set_volume(volume);
        }
    }
}

impl Component for Sound {
    fn start(&mut self, ctx: &mut impl EngineApi, _base: &mut Base) {
        if self.auto_play {
            self.play(ctx);
        }
    }
    fn destroy(&mut self, _ctx: &mut impl EngineApi, _base: &Base) {
        self.stop();
    }
}
