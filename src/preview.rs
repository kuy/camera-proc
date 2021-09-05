use crate::vertex::Vertex;
use asdf_pixel_sort::sort;
use glium::{
    implement_vertex, index::PrimitiveType, program, texture::RawImage2d, uniform, Display,
    IndexBuffer, Surface, Texture2d, VertexBuffer,
};
use glutin::platform::unix::EventLoopExtUnix;
use glutin::{event_loop::EventLoop, window::WindowBuilder, ContextBuilder};
use image::RgbImage;
use std::{
    thread::{self, JoinHandle},
    time::Instant,
};

pub fn run(rx: flume::Receiver<RgbImage>) -> JoinHandle<()> {
    thread::spawn(move || {
        let gl_event_loop: EventLoop<()> = EventLoop::new_any_thread();
        let window_builder = WindowBuilder::new().with_title("prv | ENDNAUT");
        let context_builder = ContextBuilder::new().with_vsync(true);
        let gl_display = Display::new(window_builder, context_builder, &gl_event_loop).unwrap();

        implement_vertex!(Vertex, position, tex_coords);

        let vert_buffer = VertexBuffer::new(
            &gl_display,
            &[
                Vertex {
                    position: [-1.0, -1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [-1.0, 1.0],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                    tex_coords: [1.0, 1.0],
                },
                Vertex {
                    position: [1.0, -1.0],
                    tex_coords: [1.0, 0.0],
                },
            ],
        )
        .unwrap();

        let idx_buf = IndexBuffer::new(
            &gl_display,
            PrimitiveType::TriangleStrip,
            &[1 as u16, 2, 0, 3],
        )
        .unwrap();

        let program = program!(&gl_display,
            140 => {
                vertex: "
        #version 140
        uniform mat4 matrix;
        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;
        void main() {
            gl_Position = matrix * vec4(position, 0.0, 1.0);
            v_tex_coords = tex_coords;
        }
    ",

                fragment: "
        #version 140
        uniform sampler2D tex;
        in vec2 v_tex_coords;
        out vec4 f_color;
        void main() {
            f_color = texture(tex, v_tex_coords);
        }
    "
            },
        )
        .unwrap();

        gl_event_loop.run(move |event, _window, ctrl| {
            if rx.len() > 3 {
                log::warn!("Lots of messages: len={}", rx.len());
                for _ in 1..rx.len() {
                    rx.recv().unwrap(); // drop frame
                }
                log::warn!("Cleared: len={}", rx.len());
            }

            let before_capture = Instant::now();
            let mut frame = rx.recv().unwrap();
            let after_capture = Instant::now();

            let before_sort = Instant::now();
            sort(&mut frame, 16);
            let after_sort = Instant::now();

            let width = &frame.width();
            let height = &frame.height();

            let raw_data = RawImage2d::from_raw_rgb(frame.into_raw(), (*width, *height));
            let gl_texture = Texture2d::new(&gl_display, raw_data).unwrap();

            let uniforms = uniform! {
                matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, -1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0f32]
                ],
                tex: &gl_texture
            };

            let mut target = gl_display.draw();
            target.clear_color(0.0, 0.0, 0.0, 0.0);
            target
                .draw(
                    &vert_buffer,
                    &idx_buf,
                    &program,
                    &uniforms,
                    &Default::default(),
                )
                .unwrap();
            target.finish().unwrap();

            match event {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => {
                        *ctrl = glutin::event_loop::ControlFlow::Exit;
                    }
                    _ => {}
                },
                _ => {}
            }

            log::debug!(
                "Took {}ms to capture, {}ms to sort",
                after_capture.duration_since(before_capture).as_millis(),
                after_sort.duration_since(before_sort).as_millis(),
            )
        })
    })
}
