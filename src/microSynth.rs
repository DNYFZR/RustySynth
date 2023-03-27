// microSynth

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use vizia::prelude::*;

static THEME: &'static str = include_str!("theme.css");

// App controller comms
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Message {
    Frequency(f32),
    Amplitude(f32),
    Note(f32),
}

// Controller widget
#[derive(Lens)]
struct AppData {
    command_sender: crossbeam_channel::Sender<Message>,
    amplitude: f32,
    frequency: f32,
}

pub enum AppEvent {
    SetAmplitude(f32),
    SetFrequency(f32),
}

impl Model for AppData {
    fn event(&mut self, _: &mut EventContext, event: &mut Event) {
        // Respond to app events
        event.map(|app_event, _| match app_event {
            
            AppEvent::SetAmplitude(amp) => {
                self.amplitude = *amp;
                self.command_sender.send(Message::Amplitude(self.amplitude)).unwrap();
            }

            AppEvent::SetFrequency(freq) => {
                self.frequency = *freq;
                self.command_sender.send(Message::Frequency(self.frequency)).unwrap();
            }
        });

        // Respond to window events
        event.map(|window_event, _| match window_event {

            WindowEvent::KeyDown(code, _ ) if *code == Code::KeyZ => {
                self.command_sender.send(Message::Note(1.0)).unwrap();
            }

            WindowEvent::KeyUp(code, _ ) if *code == Code::KeyZ => {
                self.command_sender.send(Message::Note(0.0)).unwrap();
            }

            _ => {}
        })
    }
}

impl AppData {
    pub fn new(command_sender: crossbeam_channel::Sender<Message>) -> Self {
        Self {
            command_sender,
            amplitude: 0.1,
            frequency: 0.0
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
        
        // Row of control knobs
        HStack::new(cx, |cx|{
            
            // Amplitude
            VStack::new(cx, |cx| {
            
                Knob::new(cx, 0.5, AppData::amplitude, false)
                    .on_changing(|cx, val| cx.emit(AppEvent::SetAmplitude(val)));
            
                Label::new(cx, AppData::amplitude.map(|amp| format!("{:.2}", amp)));
            
            }).class("control");
            
            // Frequency
            VStack::new(cx, |cx| {
                
                Knob::new(cx, 0.0, AppData::frequency, false)
                    .on_changing(|cx, val| cx.emit(AppEvent::SetFrequency(val)));
                
                Label::new(cx, AppData::frequency.map(|freq| format!("{:.0} Hz", 440.0 + *freq * (2000.0 - 440.0))), );
            })
            .class("control");
        })
        .class("content");

    })
    .title("RustySynth")
    .inner_size((200, 120))
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
    let mut frequency = 440.0f32;
    let mut amplitude = 0.1;
    let mut note = 0.0;


    // Buidl output stream
    let stream = device.build_output_stream(
        config, 
        move | data: &mut [T], _: &cpal::OutputCallbackInfo | {
            
            // Frame buffer for each channel (curr = 2)
            for frame in data.chunks_mut(channels) {

                // While input from GUI thread
                while let Ok(command) = command_receiver.try_recv() {
                    match command {

                        Message::Note(val) => {
                            note = val;
                        }

                        Message::Amplitude(val) => {
                            amplitude = val;
                        }

                        Message::Frequency(val) => {
                            frequency = (val * (2000.0 - 440.0)) + 440.0;
                        }
                    }
                }
                // Create phase clock
                phi = (phi + (frequency / sample_rate)).fract();

                let make_noise = |phi: f32| -> f32 {amplitude * note * (2.0f32 * 3.141592f32 * phi).sin()};

                // Convert make_noise output to sample
                let value: T = cpal::Sample::from::<f32>(&make_noise(phi));

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
