use std::sync::mpsc;
use std::thread;
use crate::config::Config;

pub enum Event {
    Push(Vec<f32>),
    Consume(mpsc::Sender<Vec<f32>>),
}

pub fn init(
    receiver: mpsc::Receiver<Event>,
    config: Config,
) {
    let mut buffer: Vec<f32> = Vec::new();
    thread::spawn(move || loop {
        match receiver.recv() {
            Ok(event) => match event {
                Event::Push(mut n) => {
                    n = smooth_buffer(buffer.clone(), n.clone());
                    bar_reduction(&mut n, config.processing.bar_reduction);
                    buffer = buffer_gravity(buffer, n, (config.processing.gravity * 0.25 ) + 1.0)
                }
                Event::Consume(sender) => {
                    sender.send(buffer.clone()).unwrap();
                }
            },
            Err(e) => eprintln!(
                "an error occured while transmitting data between threads, {}",
                e
            ),
        };
    });
}

fn buffer_gravity(
    mut old_buffer: Vec<f32>,
    new_buffer: Vec<f32>,
    gravity: f32,
) -> Vec<f32> {
    // buffering and time smoothing
    let mut output_buffer: Vec<f32> = Vec::new();
    let difference: i32 = new_buffer.len() as i32 - old_buffer.len() as i32;
    if difference > 0 {
        for _ in 0..difference {
            old_buffer.push(0.0);
        }
    }
    for i in 0..new_buffer.len() {
        if new_buffer[i] > old_buffer[i] {
            old_buffer[i] = new_buffer[i]
        }
        output_buffer.push(old_buffer[i] / gravity);
    }
    return output_buffer;
}

fn smooth_buffer(
    mut old_buffer: Vec<f32>,
    new_buffer: Vec<f32>,
) -> Vec<f32> {
    let mut output_buffer: Vec<f32> = Vec::new();
    let difference: i32 = new_buffer.len() as i32 - old_buffer.len() as i32;
    if difference > 0 {
        for _ in 0..difference {
            old_buffer.push(0.0);
        }
    }
    for i in 0..new_buffer.len() {
        output_buffer.push((old_buffer[i] + new_buffer[i]) / 2.0);
    }
    output_buffer
}

pub fn bar_reduction(buffer: &mut Vec<f32>, bar_reduction: f32) {
    // reduces number of bars, but keeps frequencies
    if bar_reduction > 0.0 {
        let mut reduce_pos: f32 = 0.0;
        loop {
            if buffer.len() > reduce_pos as usize {
                buffer.remove(reduce_pos as usize);
            } else {
                break;
            }
            reduce_pos += bar_reduction
        }
    }
}
