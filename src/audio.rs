use rodio::{OutputStream, Sink, Source, source::SquareWave};

pub struct Audio {
    sink: Sink,
    _stream_handle: OutputStream // If the stream gets dropped the audio stops
}

impl Audio {
   pub fn new(freq: f32) -> Self {
       let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
           .expect("open default audio stream");
       let sink = rodio::Sink::connect_new(&stream_handle.mixer());

       let tone = SquareWave::new(freq).amplify(0.85);
       sink.append(tone.repeat_infinite());

       sink.pause();

       Self { sink, _stream_handle: stream_handle }
   }

   pub fn start(&self) {
       if self.sink.is_paused() {
           self.sink.play();
       }
    }

    pub fn stop(&self) {
        if !self.sink.is_paused() {
            self.sink.pause();
        }
    } 
}
