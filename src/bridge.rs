use std::sync::mpsc;
use std::thread;
use rustfft::{FftPlanner, num_complex::Complex};

pub enum Event {
    Push(Vec<f32>),
    Consume(mpsc::Sender<Vec<f32>>),
}


pub fn init(receiver: mpsc::Receiver<Event>) {
    let mut buffer: Vec<f32> = vec![0.0];
    thread::spawn(move || loop {
        match receiver.recv() {
            Ok(event) => {
                match event {
                    Event::Push(mut n) => {
                        let n = convert_buffer(n);
                        buffer = n;
                    }
                    Event::Consume(sender) => {
                        sender.send(buffer.clone()).unwrap()
                    }
                }
            },
            Err(_) => (),
        };
    });
}

pub fn convert_buffer(input_buffer: Vec<f32>) -> Vec<f32> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(input_buffer.len());

    let mut buffer: Vec<Complex<f32>> = Vec::new();
    for i in 0..input_buffer.len() {
        buffer.push(Complex {  re: input_buffer[i], im: 0.0 });
    }
    fft.process(&mut buffer[..]);

    let mut output_buffer: Vec<f32> = Vec::new();
    for i in buffer.iter() {
        output_buffer.push(i.norm())
    }
    output_buffer
}
