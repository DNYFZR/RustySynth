// drumPad

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use vizia::prelude::*;

static THEME: &'static str = include_str!("theme.css");

// App controller comms
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Message {
    Pad(f32)
}

// Controller widget
#[derive(Lens)]
struct AppData {
    command_sender: crossbeam_channel::Sender<Message>,
    pad: f32,
}

pub enum AppEvent {
    DrumPad1(f32),
    DrumPad2(f32),
    DrumPad3(f32),
    DrumPad4(f32),
    DrumPad5(f32),
    DrumPad6(f32),
    DrumPad7(f32),
    DrumPad8(f32),
}

impl Model for AppData {
    fn event(&mut self, _: &mut EventContext, event: &mut Event) {
        // Respond to app events
        event.map(|app_event, _| match app_event {
    
            AppEvent::DrumPad1(_pad) => {
                self.pad = 1.0;     
            }

            AppEvent::DrumPad2(_pad) => {
                self.pad = 2.0;
            }

            AppEvent::DrumPad3(_pad) => {
                self.pad = 3.0;      
            }

            AppEvent::DrumPad4(_pad) => {
                self.pad = 4.0;      
            }

            AppEvent::DrumPad5(_pad) => {
                self.pad = 5.0;      
            }

            AppEvent::DrumPad6(_pad) => {
                self.pad = 6.0;      
            }

            AppEvent::DrumPad7(_pad) => {
                self.pad = 7.0;      
            }

            AppEvent::DrumPad8(_pad) => {
                self.pad = 8.0;      
            }
        });

        // Respond to window events
        event.map(|window_event, _| match window_event {

            WindowEvent::KeyDown(code, _ ) if *code == Code::KeyB => {
                self.command_sender.send(Message::Pad(1.0)).unwrap();
            }

            WindowEvent::KeyDown(code, _ ) if *code == Code::KeyS => {
                self.command_sender.send(Message::Pad(2.0)).unwrap();
            }

            _ => {}
        })
    }
}

impl AppData {
    pub fn new(command_sender: crossbeam_channel::Sender<Message>) -> Self {
        Self {
            command_sender,
            pad: 0.0,
        }
    }
}

fn main() {
    // Crossbeam channel for i/o
    let (command_sender, command_receiver) = crossbeam_channel::bounded(1024);
    
    // Audio threads
    std::thread::spawn(move || {
        
        let host = cpal::default_host();

        let device = host.default_output_device().expect("failed to locate audio output device");

        let config = device.default_output_config().unwrap();

        match config.sample_format() {
            cpal::SampleFormat::F32 => {
                run::<f32>(&device, &config.into(), command_receiver.clone()).unwrap();
            }

            cpal::SampleFormat::I16 => {
                run::<i16>(&device, &config.into(), command_receiver.clone()).unwrap();
            }

            cpal::SampleFormat::U16 => {
                run::<u16>(&device, &config.into(), command_receiver.clone()).unwrap();
            }
        }

    });

    // User interface
    Application::new(move |cx|{
        cx.add_theme(THEME);

        AppData::new(command_sender.clone()).build(cx);
         
        // Drum Pad
        VStack::new(cx, |cx| {
            Label::new(cx, "Drum Pad", );
            
            // top row
            HStack::new(cx, |cx| {
                Button::new(cx, |cx| cx.emit(AppEvent::DrumPad1(1.0)), |cx| Label::new(cx, "1"));
                Button::new(cx, |_| {}, |cx| Label::new(cx, "2"));
                Button::new(cx, |_| {}, |cx| Label::new(cx, "3"));
                Button::new(cx, |_| {}, |cx| Label::new(cx, "4"));
            
            }).class("control");
            
            // bottom row
            HStack::new(cx, |cx| {
                Button::new(cx, |_| {}, |cx| Label::new(cx, "5"));
                Button::new(cx, |_| {}, |cx| Label::new(cx, "6"));
                Button::new(cx, |_| {}, |cx| Label::new(cx, "7"));
                Button::new(cx, |_| {}, |cx| Label::new(cx, "8"));
            
            }).class("control");
            
        }).class("content");

    })
    .title("RustySynth")
    .inner_size((800, 600))
    .run();
}


// Run method
fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, command_receiver: crossbeam_channel::Receiver<Message>) -> Result<(), anyhow::Error> 
where 
    T: cpal::Sample, {

    // Get the sample rate & channel number
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let err_fn = |err| eprintln!("Stream Error : {}", err);

    // Define oscillator vars
    let mut phi = 0.0f32;
    let mut pad = 0.0;


    // Buidl output stream
    let stream = device.build_output_stream(
        config, 
        move | data: &mut [T], _: &cpal::OutputCallbackInfo | {
            
            // Frame buffer for each channel (curr = 2)
            for frame in data.chunks_mut(channels) {

                // While input from GUI thread
                while let Ok(command) = command_receiver.try_recv() {
                    match command {

                        Message::Pad(val) => {
                            pad = val;
                        }
                    }
                }
                
                // Convert make_noise output to sample
                let value: T = cpal::Sample::from::<f32>("src/data/Roland TR-808/BD/BD0000.WAV");

                for sample in frame.iter_mut() {
                    *sample = value;
                }
            }
        } , 
        err_fn,
    )?;

    // Play stream
    stream.play()?;

    // Park the thread for continuous playback
    std::thread::park();

    // Return ok
    Ok(())
}

