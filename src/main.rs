// RustySynth

use cpal;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    // Crossbeam channel for i/o
    let (command_sender, command_receiver) = crossbeam_channel::bounded::<i32>(1024);
    
    // Audio threads
    std::thread::spawn(move || {
        
        let host = cpal::default_host();

        let device = host.default_output_device().expect("failed to locate audio output device");

        let config = device.default_output_config().unwrap();
        

        match config.sample_format() {
            cpal::SampleFormat::F32 => {
                // run::<f32>(&device, &config.into(), command_receiver.clone()).unwrap();
            }

            cpal::SampleFormat::I16 => {
                // run::<i16>(&device, &config.into(), command_receiver.clone()).unwrap();
            }

            cpal::SampleFormat::U16 => {
                // run::<u16>(&device, &config.into(), command_receiver.clone()).unwrap();
            }
        }
    });


}

