use rodio::{DeviceSinkBuilder, MixerDeviceSink};

pub struct Audio {
    sink: MixerDeviceSink,
}

impl Audio {
    fn new() -> Self {
        let stream_handle = DeviceSinkBuilder::open_default_sink().unwrap();
        Self {
            sink: stream_handle,
        }
    }
}
