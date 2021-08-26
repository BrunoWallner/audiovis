use crate::graphics::wgpu_abstraction::Vertex;

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

                //let top_color: [f32; 3] = [ top_color[0] * (y + 1.0),  top_color[1] * (y + 1.0),  top_color[2] * (y + 1.0)];

                /*
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
                */
                vertices.push(Vertex { position: [x - width,  -1.0, 0.35],   color:  bottom_color });
                vertices.push(Vertex { position: [x - width,  -1.0, 0.25],   color:  bottom_color });
                vertices.push(Vertex { position: [x + width,  -1.0, 0.25],   color:  bottom_color });
                vertices.push(Vertex { position: [x + width,  -1.0, 0.35],   color:  bottom_color });

                vertices.push(Vertex { position: [x - width,  y, 0.35],   color: top_color });
                vertices.push(Vertex { position: [x - width,  y, 0.25],   color: top_color });
                vertices.push(Vertex { position: [x + width,  y, 0.25],   color: top_color });
                vertices.push(Vertex { position: [x + width,  y, 0.35],   color: top_color });

                let i = vertices.len() as u16 - 8;
                indices.push(i+0);
                indices.push(i+7);
                indices.push(i+4);
                indices.push(i+0);
                indices.push(i+3);
                indices.push(i+7);

                indices.push(i+1);
                indices.push(i+4);
                indices.push(i+5);
                indices.push(i+1);
                indices.push(i+0);
                indices.push(i+4);

                indices.push(i+2);
                indices.push(i+5);
                indices.push(i+6);
                indices.push(i+2);
                indices.push(i+1);
                indices.push(i+5);

                indices.push(i+3);
                indices.push(i+6);
                indices.push(i+7);
                indices.push(i+3);
                indices.push(i+2);
                indices.push(i+6);

                indices.push(i+0);
                indices.push(i+1);
                indices.push(i+2);
                indices.push(i+0);
                indices.push(i+2);
                indices.push(i+3);

                indices.push(i+4);
                indices.push(i+6);
                indices.push(i+5);
                indices.push(i+4);
                indices.push(i+7);
                indices.push(i+6);
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

