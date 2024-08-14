// gstreamer_backend.rs
use gst::prelude::*;
use gst::{Element, Pipeline, State};
use gstreamer as gst;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

use super::AudioBackend;
use crate::{
    stream::{PCMState, Stream},
    virtio_sound::{self, VirtioSndPcmSetParams},
    Direction, Error, Result as CrateResult,
};

#[derive(Clone)]
pub struct GStreamerBackend {
    pipelines: Vec<Arc<Mutex<Pipeline>>>,
    streams: Arc<RwLock<Vec<Stream>>>,
}

impl std::fmt::Debug for GStreamerBackend {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("GStreamerBackend")
            .field("pipelines_no", &self.pipelines.len())
            .finish_non_exhaustive()
    }
}

impl GStreamerBackend {
    fn setup_pipeline(&self, stream_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        let streams = self.streams.read().unwrap();
        let s = &streams[stream_id];

        gst::init()?;

        let pipeline = gst::Pipeline::new();

        let source = gst::ElementFactory::make("appsrc").build().unwrap();
        let convert = gst::ElementFactory::make("audioconvert").build().unwrap();
        let resample = gst::ElementFactory::make("audioresample").build().unwrap();
        let sink = gst::ElementFactory::make("autoaudiosink").build().unwrap();

        pipeline.add_many(&[&source, &convert, &resample, &sink])?;
        gst::Element::link_many(&[&source, &convert, &resample, &sink])?;

        let caps = gst::Caps::builder("audio/x-raw")
            .field("channels", &s.params.channels)
            .field("rate", &self.get_rate(s.params.rate))
            .field("format", &"S16LE")
            .build();

        source.set_property("caps", &caps).unwrap();

        pipeline.set_state(gst::State::Playing)?;

        self.pipelines[stream_id] = Arc::new(Mutex::new(pipeline));

        Ok(())
    }

    fn get_rate(&self, rate: u8) -> i32 {
        match rate {
            virtio_sound::VIRTIO_SND_PCM_RATE_5512 => 5512,
            virtio_sound::VIRTIO_SND_PCM_RATE_8000 => 8000,
            virtio_sound::VIRTIO_SND_PCM_RATE_11025 => 11025,
            virtio_sound::VIRTIO_SND_PCM_RATE_16000 => 16000,
            virtio_sound::VIRTIO_SND_PCM_RATE_22050 => 22050,
            virtio_sound::VIRTIO_SND_PCM_RATE_32000 => 32000,
            virtio_sound::VIRTIO_SND_PCM_RATE_44100 => 44100,
            virtio_sound::VIRTIO_SND_PCM_RATE_48000 => 48000,
            virtio_sound::VIRTIO_SND_PCM_RATE_64000 => 64000,
            virtio_sound::VIRTIO_SND_PCM_RATE_88200 => 88200,
            virtio_sound::VIRTIO_SND_PCM_RATE_96000 => 96000,
            virtio_sound::VIRTIO_SND_PCM_RATE_176400 => 176400,
            virtio_sound::VIRTIO_SND_PCM_RATE_192000 => 192000,
            virtio_sound::VIRTIO_SND_PCM_RATE_384000 => 384000,
            _ => panic!("Unsupported rate"),
        }
    }
}
