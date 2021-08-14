use crate::*;
use rustfft::{num_complex::Complex, FftPlanner};
use std::thread;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub fn stream_input(
    bridge_sender: mpsc::Sender<bridge::Event>,
    m_freq: u32,
    pre_fft_windowing: bool
) {
    thread::spawn(move || {
        let (tx, rc) = mpsc::channel();
        instruction_receiver(rc, bridge_sender, m_freq, pre_fft_windowing);

        let host = cpal::default_host();
        let input_device = host.default_input_device().unwrap();

        println!("using input devices: {}", input_device.name().unwrap());

        let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();

        let input_data_fn =
            move |data: &[f32], _: &cpal::InputCallbackInfo| match tx.send(data.to_vec()) {
                Ok(_) => (),
                Err(e) => eprintln!("failed to send audio data to bridge, {}", e),
            };

        let input_stream = input_device
            .build_input_stream(&config, input_data_fn, err_fn)
            .unwrap();

        input_stream.play().unwrap();

        loop {}
    });
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

fn instruction_receiver(
    receiver: mpsc::Receiver<Vec<f32>>,
    sender: mpsc::Sender<bridge::Event>,
    m_freq: u32,
    pre_fft_windowing: bool,
) {
    thread::spawn(move || loop {
        match receiver.recv() {
            Ok(b) => {
                let sender_clone = sender.clone();
                thread::spawn(move || {
                    let buffer: Vec<f32> = convert_buffer(b.to_vec(), m_freq, pre_fft_windowing);
                    sender_clone.send(bridge::Event::Push(buffer)).unwrap();
                });
            }
            Err(e) => eprintln!("failed to send audio data to bridge, {}", e),
        }
    });
}

fn apodize(buffer: Vec<f32>) -> Vec<f32> {
    let window = apodize::hanning_iter(buffer.len()).collect::<Vec<f64>>();

    let mut output_buffer: Vec<f32> = Vec::new();

    for i in 0..buffer.len() {
        output_buffer.push(window[i] as f32 * buffer[i]);
    }
    output_buffer
}

pub fn convert_buffer(
    input_buffer: Vec<f32>,
    m_freq: u32, pre_fft_windowing: bool
) -> Vec<f32> {
    let mut input_buffer: Vec<f32> = input_buffer;
    if pre_fft_windowing {
        input_buffer = apodize(input_buffer)
    }

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(input_buffer.len());

    let mut buffer: Vec<Complex<f32>> = Vec::new();
    for i in 0..input_buffer.len() {
        buffer.push(Complex {
            re: input_buffer[i],
            im: 0.0,
        });
    }
    fft.process(&mut buffer[..]);

    let mut output_buffer: Vec<f32> = Vec::new();
    let length: usize = buffer.len() / 2;
    for i in 0..length as usize {
        output_buffer.push(buffer[i].norm())
    }
    // *0.425 to cut off unwanted vector information that just mirrors itself and trims to exactly 20khz
    let output_buffer = output_buffer[0..(output_buffer.len() as f32 * 0.455) as usize].to_vec();

    // max frequency
    let percentage: f32 = m_freq as f32 / 20000.0;
    output_buffer[0..(output_buffer.len() as f32 * percentage) as usize].to_vec()
}
