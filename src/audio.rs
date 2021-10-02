use gag::Gag;
use std::thread;
use std::sync::mpsc;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[derive(Debug, Clone, Copy)]
pub enum AudioDevice {
    Input(usize),
    Output(usize),
}

pub fn init_audio_sender(event_sender: mpsc::Sender<audioviz::Event>, audio_device: AudioDevice) {
    thread::spawn(move || {
        // dont print any alsa or jack errors on *nix systems to stderr
        let _print_gag = Gag::stderr().unwrap();

        let host = cpal::default_host();
        let input_devices = host.input_devices().unwrap().collect::<Vec<cpal::Device>>();
        let output_devices = host.output_devices().unwrap().collect::<Vec<cpal::Device>>();

        let device = match audio_device {
            AudioDevice::Input(i) => {
                &input_devices[i]
            }
            AudioDevice::Output(i) => {
                &output_devices[i]
            }
        };

        match audio_device {
            AudioDevice::Input(_) => {
                println!("using input device: {}", device.name().unwrap());
            }
            AudioDevice::Output(_) => {
                println!("using output device: {}", device.name().unwrap());
            }
        }

        //let device_config =  device.default_input_config().unwrap();
        let device_config = match audio_device {
            AudioDevice::Input(_) => {
                device.default_input_config().unwrap()
            }
            AudioDevice::Output(_) => {
                device.default_output_config().unwrap()
            }
        };

        let stream = match device_config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &device_config.into(),
                move |data, _: &_| handle_input_data_f32(data, event_sender.clone()),
                err_fn,
            ).unwrap(),
            other => {
                panic!("Unsupported sample format {:?}", other);
            }
        };

        stream.play().unwrap();

        // parks the thread so stream.play() does not get dropped and stops
        thread::park();
    });
}

pub fn iter_audio_devices() {
    let input_devices: Vec<cpal::Device>;
    let output_devices: Vec<cpal::Device>;
    {
        // dont print any alsa or jack errors on *nix systems to stderr
        let _print_gag = Gag::stderr().unwrap();

        let host = cpal::default_host();
        input_devices = 
            host.input_devices()
            .unwrap()
            .collect::<Vec<cpal::Device>>();

        output_devices = 
            host.output_devices()
            .unwrap()
            .collect::<Vec<cpal::Device>>();
    }
    
    println!("[input devices]");
    for (i, x) in input_devices.iter().enumerate() {
        println!("{}: {}", i, x.name().unwrap());
    }
    println!("");

    println!("[output devices]");
    for (i, x) in output_devices.iter().enumerate() {
        println!("{}: {}", i, x.name().unwrap());
    }
}

fn handle_input_data_f32(data: &[f32], sender: mpsc::Sender<audioviz::Event>) {
    // sends the raw data to audio_stream via the event_sender
    sender.send(audioviz::Event::SendData(data.to_vec())).unwrap();
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}