use std::{fs, io::Cursor, num::NonZero, sync::Mutex};

use rodio::{
    Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source,
    mixer::{self, Mixer},
};

use crate::{core::Handler, resources::Resources};

pub struct AudioAsset {
    pub bytes: Vec<u8>,
}

impl AudioAsset {
    pub fn load_audio(path: &str) -> Self {
        Self {
            bytes: fs::read(path).unwrap(),
        }
    }
}

pub struct AudioSys {
    sink: MixerDeviceSink,
    players: Mutex<Vec<Player>>,
    _mixer: Mixer,
}

impl Default for AudioSys {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioSys {
    pub fn new() -> Self {
        let (controller, mixer) =
            mixer::mixer(NonZero::new(2).unwrap(), NonZero::new(44_100).unwrap());

        let mut sink = DeviceSinkBuilder::open_default_sink().unwrap();
        sink.log_on_drop(false);
        let player = Player::connect_new(sink.mixer());
        player.set_volume(1.0);

        player.append(mixer);

        Self {
            sink,
            players: Mutex::new(Vec::new()),
            _mixer: controller,
        }
    }
    pub fn play_controled(
        &self,
        resources: &Resources,
        sound: Handler<AudioAsset>,
        looped: bool,
    ) -> Player {
        let sound_bytes = &resources.sounds.get(sound).unwrap().bytes;
        let source = Decoder::try_from(Cursor::new(sound_bytes.to_vec())).unwrap();

        let mixer = self.sink.mixer();

        let player = Player::connect_new(mixer);
        if looped {
            player.append(source.repeat_infinite());
        } else {
            player.append(source);
        }
        player
    }

    pub fn play_and_forget(&self, resources: &Resources, sound: Handler<AudioAsset>) {
        let sound_bytes = &resources.sounds.get(sound).unwrap().bytes;
        let source = Decoder::try_from(Cursor::new(sound_bytes.to_vec())).unwrap();

        let mixer = self.sink.mixer();

        let player = Player::connect_new(mixer);
        player.append(source);

        if let Ok(mut players) = self.players.lock() {
            players.retain(|p| !p.empty());
            players.push(player);
        }
    }
}
