use crate::wgpu_abstraction::Vertex;

pub fn convert_to_buffer(
    buffer: Vec<f32>,
    visualisation: String,
    width: f32,
    volume_amplitude: f32,
    volume_factoring: f32,
    top_color: [f32; 3],
    bottom_color: [f32; 3],
) -> (Vec<Vertex>, Vec<u16>)  {

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    let buffer_len = buffer.len();
    let width: f32 = 1.0 / buffer_len as f32 *   width;

    match visualisation.as_str() {
        "Bars" => {
            for i in 0..buffer.len() {
                let x = (i as f32 - buffer_len as f32 / 2.0) / (buffer_len as f32 / 2.0) + width;
                let y: f32 = volume_amplitude * ( (buffer[i] as f32).powf(volume_factoring) * 0.05 ) - 1.0;

                let top_color: [f32; 3] = [ top_color[0] * (y + 1.0),  top_color[1] * (y + 1.0),  top_color[2] * (y + 1.0)];

                vertices.push(Vertex { position: [x - width,  -1.0, 0.0],   color:  bottom_color });
                vertices.push(Vertex { position: [x - width,  y, 0.0],   color: top_color });
                vertices.push(Vertex { position: [x + width,  y, 0.0],   color: top_color });
                vertices.push(Vertex { position: [x + width,  -1.0, 0.0],   color:  bottom_color });

                let i = vertices.len() as u16 - 4;
                indices.push(i + 2);
                indices.push(i + 1);
                indices.push(i + 0);
                indices.push(i + 2);
                indices.push(i + 0);
                indices.push(i + 3);
            }
        },
        "Strings" => {
            let width = width * 2.5;
            for i in 0..buffer.len() - 1 {
                let x1: f32 = (i as f32 - buffer_len as f32 / 2.0) / (buffer_len as f32 / 2.0);
                let x2: f32 = ((i + 1) as f32 - buffer_len as f32 / 2.0) / (buffer_len as f32 / 2.0);
                let y1: f32 = volume_amplitude * ( (buffer[i] as f32).powf(volume_factoring) * 0.05 ) - 1.0;
                let y2: f32 = volume_amplitude * ( (buffer[i + 1] as f32).powf(volume_factoring) * 0.05 ) - 1.0;

                let color: [f32; 3] = [top_color[0] * (y1 + 1.0), top_color[1] * (y1 + 1.0), top_color[2] * (y1 + 1.0)];

                let (mut vertices2, mut indices2) = draw_line([x1, y1], [x2, y2], width, color, vertices.len() as u16);
                vertices.append(&mut vertices2);
                indices.append(&mut indices2);
            }
        },
        _ => (),
    }
    return (vertices, indices);
}

fn draw_line(
    point1: [f32; 2],
    point2: [f32; 2],
    width: f32,
    color: [f32; 3],
    vertex_len: u16,
) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    let x1: f32 = point1[0];
    let x2: f32 = point2[0];
    let y1: f32 = point1[1];
    let y2: f32 = point2[1];

    let dx = x2 - x1;
    let dy = y2 - y1;
    let l = dx.hypot (dy);
    let u = dx * width * 0.5 / l;
    let v = dy * width * 0.5 / l;

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

fn get_char_data(c: char) -> Vec<[f32; 2]> {
    match c {
        'A' => vec![ [0.1, 0.0], [0.25, 1.0], [0.25, 1.0], [0.75, 1.0], [0.75, 1.0], [0.9, 0.0], [0.175, 0.5], [0.825, 0.5] ],
        'B' => vec![ [0.2, 0.0], [0.2, 1.0], [0.2, 1.0], [0.7, 1.0], [0.7, 1.0], [0.8, 0.9], [0.8, 0.9], [0.8, 0.6], [0.8, 0.6], [0.7, 0.5], [0.7, 0.5], [0.2, 0.5], [0.7, 0.5], [0.8, 0.4], [0.8, 0.4], [0.8, 0.1], [0.8, 0.1], [0.7, 0.0], [0.7, 0.0], [0.2, 0.0] ],

        'P' => vec![ [0.2, 0.0], [0.2, 1.0], [0.2, 1.0], [0.7, 1.0], [0.7, 1.0], [0.8, 0.9], [0.8, 0.9], [0.8, 0.6], [0.8, 0.6], [0.7, 0.5], [0.7, 0.5], [0.2, 0.5] ],
        _ => vec![ [0.2, 0.0], [0.8, 0.0], [0.8, 0.0], [0.8, 1.0], [0.8, 1.0], [0.2, 1.0], [0.2, 1.0], [0.2, 0.0] ],
    }
}

pub fn convert_text(text: String, width: f32, scale: [f32; 2], color: [f32; 3], vertex_len: u16) -> ([f32; 2], (Vec<Vertex>, Vec<u16>)) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    let mut x: f32 = 0.0;
    let mut y: f32 = 0.0;

    for c in text.chars() {
        match c {
            '\n' => {
                x = 0.0;
                y -= 1.25;
            },
            ' ' => {
                x += 1.0;
            }
            _ => {
                let lines = get_char_data(c);
                for i in 0..lines.len() / 2 {
                    let i = i*2;
                    let (mut c_v, mut c_i) = draw_line(
                        [lines[i][0] + x,   lines[i][1] + y],
                        [lines[i+1][0] + x, lines[i+1][1] + y],
                        width,
                        color,
                        vertex_len + vertices.len() as u16,
                    );
                    vertices.append(&mut c_v);
                    indices.append(&mut c_i);
                }
                x += 1.0;
            }
        }
    }

    // scaling endbuffer
    for mut v in vertices.iter_mut() {
        v.position[0] *= scale[0];
        v.position[1] *= scale[1];
    }

    return ([x * scale[0], y * scale[1]], (vertices, indices));
}

