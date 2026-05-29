use rodio::Player;

use crate::{AudioAsset, Component, EngineApi, Handler};

pub struct Sound {
    sound: Handler<AudioAsset>,
    pub auto_play: bool,
    pub looped: bool,
    pub volume: f32,
    playing: Option<Player>,
    pub one_shot: bool,
}

impl Sound {
    pub fn new(sound: Handler<AudioAsset>) -> Self {
        Self {
            sound,
            auto_play: false,
            looped: false,
            volume: 1.0,
            playing: None,
            one_shot: false,
        }
    }
    pub fn play(&mut self, ctx: &mut impl EngineApi) {
        let handle = ctx.play(self.sound);
        handle.set_volume(self.volume);
        self.playing = Some(handle);
    }
    pub fn stop(&mut self) {
        if let Some(handle) = &self.playing {
            handle.stop();
        }
        self.playing = None
    }
}

impl Component for Sound {
    fn start(&mut self, ctx: &mut impl EngineApi, _base: &mut crate::Base) {
        if self.auto_play {
            self.play(ctx);
        }
    }
    fn destroy(&mut self, _ctx: &mut impl EngineApi, _base: &crate::Base) {
        self.stop();
    }
}
