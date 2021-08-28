use std::sync::mpsc;
use std::thread;
use crate::config::Config;

pub enum Event {
    PushReduced(Vec<Vec<f32>>),
    Push(Vec<f32>),
    Consume(mpsc::Sender<Vec<Vec<f32>>>),
}

pub fn init(
    receiver: mpsc::Receiver<Event>,
    sender: mpsc::Sender<Event>,
    config: Config,
) {
    let mut reduced_buffer: Vec<Vec<f32>> = Vec::new();
    let mut buffer: Vec<Vec<f32>> = Vec::new();
    let mut smoothing_buffer: Vec<f32> = Vec::new();
    thread::spawn(move || loop {
        match receiver.recv() {
            Ok(event) => match event {
                Event::Push(mut n) => {
                    bar_reduction(&mut n, config.processing.bar_reduction);
                    if buffer.len() > 0 {
                        n = smooth_buffer(smoothing_buffer.clone(), n.clone(), config.visual.smoothing_amount, config.visual.smoothing_size);
                        n = buffer_gravity(smoothing_buffer.clone(), n, (config.processing.gravity * 0.25 ) + 1.0);
                    }
                    smoothing_buffer = n.clone();
                    buffer.insert(0, n);
                    if buffer.len() > config.processing.buffering as usize {
                        buffer.pop();
                    }
                    let mut buffer = buffer.clone();
                    let config = config.clone();
                    let sender = sender.clone();
                    thread::spawn(move || {
                        reduce_buffer(&mut buffer, config.processing.buffer_resolution_drop, config.processing.max_buffer_resolution_drop);
                        sender.send(Event::PushReduced(buffer)).unwrap();
                    });
                }
                Event::Consume(sender) => {
                    sender.send(reduced_buffer.clone()).unwrap();
                }
                Event::PushReduced(rb) => {
                    reduced_buffer = rb;
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
    smoothing: u32,
    smoothing_size: u32,
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
    for _ in 0..smoothing {
        for i in 0..output_buffer.len() - smoothing_size as usize {
            let mut y = 0.0;
            for x in 0..smoothing_size as usize {
                y += output_buffer[i+x];
            }
            output_buffer[i] = y / smoothing_size as f32;
        }
    }
    output_buffer
}

pub fn bar_reduction(buffer: &mut Vec<f32>, bar_reduction: u32) {
    // reduces number of bars, but keeps frequencies
    for _ in 0..bar_reduction {
        let mut pos: usize = 0;
        loop {
            if buffer.len() > pos + 1 {
                if buffer[pos] < buffer[pos+1] {
                    buffer.remove(pos);
                } else {
                    buffer.remove(pos+1);
                }
                pos += 2;
            } else {
                break;
            }
        }
    }
}

fn reduce_buffer(buffer: &mut Vec<Vec<f32>>, resolution_drop: f32, max_res_drop: u16) {
    for z in 0..buffer.len() {
        if resolution_drop > 0.0 {
            let mut amount = (z as f32 * resolution_drop * 0.1) as usize;
            if amount > max_res_drop as usize {
                amount = max_res_drop as usize;
            }
            for _ in 0..amount {
                let mut pos: usize = 1; // potential shifting but idk
                loop {
                    if buffer[z].len() > pos + 1 {
                        if buffer[z][pos] < buffer[z][pos+1] {
                            buffer[z].remove(pos);
                        } else {
                            buffer[z].remove(pos+1);
                        }
                        pos += 2;
                    } else {
                        break;
                    }
                }
            }
        }
    }
}
