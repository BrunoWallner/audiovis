use std::sync::mpsc;
use std::thread;
use std::time::{Instant, Duration};
use ringbuf::RingBuffer;

pub enum Event {
    Push(Vec<f32>),
    Consume(mpsc::Sender<Vec<f32>>),
}


pub fn init(receiver: mpsc::Receiver<Event>, buffering: usize, smooth_size: u32, smooth_amount: u32) {
    let mut buffer: Vec<Vec<f32>> = Vec::new();
    let mut rounded_buffer: Vec<f32> = Vec::new();
    let mut state: f32 = 0.0;
    thread::spawn(move || loop {
        match receiver.recv() {
            Ok(event) => {
                match event {
                    Event::Push(mut n) => {
                        // 500 µs
                        n = smooth_buffer(&mut buffer, n, buffering, smooth_size, smooth_amount);
                        rounded_buffer = n;
                    }
                    Event::Consume(sender) => {
                        sender.send(rounded_buffer.clone()).unwrap();
                    }
                }
            },
            Err(_) => (),
        };
    });
}


pub fn smooth_buffer(input_buffer: &mut Vec<Vec<f32>>, new_buffer: Vec<f32>, max_buffers: usize, smooth_size: u32, smooth_amount: u32) -> Vec<f32> {;

    /*
    for input_buffer in input_buffer.iter() {
        if new_buffer.len() < input_buffer.len() || new_buffer.len() == input_buffer.len() {
            for i in 0..new_buffer.len() {
                buffer.push( (new_buffer[i] + input_buffer[i]) / 2.0);
            }
        }
        if new_buffer.len() > input_buffer.len() {
            for i in 0..input_buffer.len() {
                buffer.push( (new_buffer[i] + input_buffer[i]) / 2.0 );
            }
            for i in input_buffer.len()..new_buffer.len() {
                buffer.push(new_buffer[i]);
            }
        }

        for _ in 0..amount {
            for i in 0..buffer.len() - 1 {
                let y = (buffer[i] + buffer[i+1]) / 2.0;
                buffer[i] = y;
            }
        }
    }
    */
    // buffering and time smoothing
    let mut output_buffer: Vec<f32> = Vec::new();
    input_buffer.insert(0, new_buffer);

    //input_buffer.push(new_buffer);
    if input_buffer.len() > max_buffers {
        input_buffer.pop();
    }

    let mut smooth_buffer: Vec<f32> = vec![0.0; 5000];
    let mut smooth_buffer_length: usize = 0;
    for buffer in input_buffer.iter() {
        if smooth_buffer_length < buffer.len() {
            smooth_buffer_length = buffer.len()
        }
        for i in 0..buffer.len() {
            if smooth_buffer_length > i {
                //output_buffer[i] = (output_buffer[i] + buffer[i]) / 2.0;
                smooth_buffer[i] += (buffer[i]);
            }
        }

    }

    for i in 0..smooth_buffer_length {
        output_buffer.push(smooth_buffer[i] / input_buffer.len() as f32);
    }

    input_buffer.pop();
    input_buffer.insert(0, output_buffer.clone());

    // horizontal smoothing
    for _ in 0..smooth_amount {
        for j in 0..output_buffer.len() - smooth_size as usize {
            let mut y: f32 = 0.0;
            for x in 0..smooth_size {
                y += output_buffer[j as usize + x as usize];
            }
            output_buffer[j] = y / smooth_size as f32;
        }
    }

    output_buffer
}
