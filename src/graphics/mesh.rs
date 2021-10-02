use crate::graphics::wgpu_abstraction::Vertex;
use crate::config::Visualisation;
use std::f32::consts::PI;

pub fn from_buffer(
    buffer: Vec<f32>,
    visualisation: Visualisation,
    width: f32,
    volume_amplitude: f32,
    volume_factoring: f32,
    top_color: [f32; 3],
    bottom_color: [f32; 3],
    size: [f32; 2],
) -> (Vec<Vertex>, Vec<u32>)  {

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let buffer_len = buffer.len();

    if buffer.len() == 0 {
        return (Vec::new(), Vec::new());
    }

    match visualisation {
        Visualisation::Bars => {
            let width: f32 = 1.0 / buffer_len as f32 * width;
            for i in 0..buffer.len() {
                let x = (i as f32 - buffer_len as f32 / 2.0) / (buffer_len as f32 / 2.0) + width;
                let y: f32 = volume_amplitude * ( (buffer[i] as f32).powf(volume_factoring) ) - 1.0;

                let top_color: [f32; 3] = [ top_color[0] * (y + 1.0),  top_color[1] * (y + 1.0),  top_color[2] * (y + 1.0)];

                vertices.push(Vertex { position: [x - width,  -1.0, 0.0],   color:  bottom_color });
                vertices.push(Vertex { position: [x + width,  -1.0, 0.0],   color:  bottom_color });

                vertices.push(Vertex { position: [x - width,  y, 0.0],   color: top_color });
                vertices.push(Vertex { position: [x + width,  y, 0.0],   color: top_color });

                let i = vertices.len() as u32 - 4;
                indices.push(i+0);
                indices.push(i+3);
                indices.push(i+2);
                indices.push(i+0);
                indices.push(i+1);
                indices.push(i+3);
            }
        },
        Visualisation::Strings => {
            let width = width * 0.005;
            for i in 0..buffer.len() - 1 {
                let x1: f32 = (i as f32 - buffer_len as f32 / 2.0) / (buffer_len as f32 / 2.0);
                let x2: f32 = ((i + 1) as f32 - buffer_len as f32 / 2.0) / (buffer_len as f32 / 2.0);
                let y1: f32 = volume_amplitude * ( (buffer[i] as f32).powf(volume_factoring)) - 1.0;
                let y2: f32 = volume_amplitude * ( (buffer[i + 1] as f32).powf(volume_factoring)) - 1.0;

                let color: [f32; 3] = [top_color[0] * (y1 + 1.0), top_color[1] * (y1 + 1.0), top_color[2] * (y1 + 1.0)];

                let (mut vertices2, mut indices2) = draw_line(
                    [x1, y1], 
                    [x2, y2], 
                    width, color, 
                    vertices.len() as u32, 
                    size,
                );
                vertices.append(&mut vertices2);
                indices.append(&mut indices2);
            }
        },
        Visualisation::Circle => {
            let width = width * 0.005;
            let radius: f32 = 0.3;
            let mut last_x: f32 = 0.0;
            let mut last_y: f32 = 0.0;

            for i in 0..buffer.len() - 1 {
                let mut angle: f32 = 2.0 * PI * (i + 1) as f32 / (buffer.len() - 2) as f32;
                let degree: f32 = 2.0 * PI / 360.0;
                angle += degree * 270.0; // rotate circle 270°

                let value: f32 = buffer[i];

                let x: f32 = angle.cos() * (value + radius) / size[0];
                let y: f32 = angle.sin() * (value + radius) / size[1];

                let r: f32 = (top_color[0] * value) + (bottom_color[0] * (1.0 / value));
                let g: f32 = (top_color[1] * value) + (bottom_color[1] * (1.0 / value));
                let b: f32 = (top_color[2] * value) + (bottom_color[2] * (1.0 / value));

                let color: [f32; 3] = [r, g ,b];

                if i != 0 {
                    let (mut vertices2, mut indices2) = draw_line(
                        [last_x, last_y], 
                        [x, y], 
                        width, 
                        color, 
                        vertices.len() as u32, 
                        size
                    );
                    vertices.append(&mut vertices2);
                    indices.append(&mut indices2);
                }
                last_x = x;
                last_y = y;
            }
        },
    }
    return (vertices, indices);
}

fn draw_line(
    point1: [f32; 2],
    point2: [f32; 2],
    width: f32,
    color: [f32; 3],
    vertex_len: u32,
    size: [f32; 2],
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let x1: f32 = point1[0];
    let x2: f32 = point2[0];
    let y1: f32 = point1[1];
    let y2: f32 = point2[1];

    let dx = x2 - x1;
    let dy = y2 - y1;
    let l = dx.hypot (dy);
    let u = dx * width * 0.5 / l / size[1];
    let v = dy * width * 0.5 / l / size[0];

    vertices.push(Vertex { position: [x1 + v,  y1 - u, 0.0], color });
    vertices.push(Vertex { position: [x1 - v,  y1 + u, 0.0], color });
    vertices.push(Vertex { position: [x2 - v,  y2 + u, 0.0], color });
    vertices.push(Vertex { position: [x2 + v,  y2 - u, 0.0], color });

    indices.push(vertex_len + 2);
    indices.push(vertex_len + 1);
    indices.push(vertex_len + 0);
    indices.push(vertex_len + 2);
    indices.push(vertex_len + 0);
    indices.push(vertex_len + 3);

    return (vertices, indices);
}

