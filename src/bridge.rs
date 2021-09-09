use std::sync::mpsc;
use std::thread;
use crate::config::Config;
use crate::graphics::mesh;

pub enum Event {
    PushMesh(mesh::Mesh),
    Push(Vec<f32>),
    Consume(mpsc::Sender<mesh::Mesh>),
}

pub fn init(
    receiver: mpsc::Receiver<Event>,
    sender: mpsc::Sender<Event>,
    config: Config,
) {
    let mut buffer: Vec<Vec<f32>> = Vec::new();
    let mut mesh: mesh::Mesh = mesh::Mesh::new();
    let mut smoothing_buffer: Vec<f32> = Vec::new();
    thread::spawn(move || loop {
        match receiver.recv() {
            Ok(event) => match event {
                Event::Push(mut n) => {
                    bar_reduction(&mut n, config.processing.bar_reduction);
                    if buffer.len() > 0 {
                        //smooth_buffer(&mut n, config.visual.smoothing_amount, config.visual.smoothing_size);
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
                        let mesh = mesh::from_buffer(
                            buffer,
                            config.visual.width,
                            config.visual.z_width,
                            config.audio.volume_amplitude,
                            config.audio.volume_factoring,
                        );
                        sender.send(Event::PushMesh(mesh)).unwrap();
                    });
                }
                Event::Consume(sender) => {
                    //sender.send(reduced_buffer.clone()).unwrap();
                    sender.send(mesh.clone()).unwrap();
                }
                Event::PushMesh(m) => {
                    mesh = m;
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

// reduces resolution of buffer
pub fn bar_reduction(buffer: &mut Vec<f32>, bar_reduction: u32) {
    if bar_reduction == 0 {return}
    let mut position: usize = 0;

    'reducing: loop {
        // break if reached end of buffer
        if position + bar_reduction as usize >= buffer.len() {
            break 'reducing;
        }

        // smoothing of bars that are gonna be removed into the bar that stays
        let mut y: f32 = 0.0;
        for x in 0..bar_reduction as usize {
            y += buffer[position + x];
        }
        buffer[position] = y / bar_reduction as f32;

        // actual removing
        for x in 1..bar_reduction as usize {
            if position + x < buffer.len() {
                buffer.remove(position + x);
            }
        }

        position += 1;
    }

    // remove last parts of buffer that cannot easily be smoothed
    if buffer.len() > bar_reduction as usize {
        for _ in 0..bar_reduction {
            buffer.pop();
        }
    }
}

// reduces resolution of buffer in the further away it is from the camera
fn reduce_buffer(buffer: &mut Vec<Vec<f32>>, resolution_drop: f32, max_res_drop: u16) {
    for z in 0..buffer.len() {
        if resolution_drop > 0.0 {
            let mut amount = (z as f32 * resolution_drop * 0.1) as usize;
            if amount > max_res_drop as usize {
                amount = max_res_drop as usize;
            }
            for _ in 0..amount {
                let mut pos: usize = 1; // to compensate for space distortion
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
