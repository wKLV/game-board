use glium;
use game::*;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    nones: [f32; 2],
}
implement_vertex!(Vertex, nones);

struct RenderObject {
    vertices: glium::VertexBuffer<Vertex>,
    indices:  glium::index::NoIndices,
    // uniforms: glium::uniforms::Uniforms,
    program:  glium::Program,
}

pub struct RenderTarget {
    display: glium::backend::glutin_backend::GlutinFacade,
    grid: RenderObject,
    entity_template: RenderObject,
    mouse_pos: (u32, u32)
}



pub const SCREEN_PIXELS_X:u32 = 800;
pub const SCREEN_PIXELS_Y:u32 = 500;

pub const CELL_SIZE_X:u32 = (SCREEN_PIXELS_X / BOARD_SIZE_X);
pub const CELL_SIZE_Y:u32 = (SCREEN_PIXELS_Y / BOARD_SIZE_Y);

pub fn init(world_board:&mut Board) -> RenderTarget {
    use glium::DisplayBuild;

    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(SCREEN_PIXELS_X as u32, SCREEN_PIXELS_Y as u32)
        .with_title(format!("tt"))
        .build_glium()
        .unwrap();


    let vertex1 = Vertex { nones: [ -1.0, -1.0] };
    let vertex2 = Vertex { nones: [  1.0, -1.0] };
    let vertex3 = Vertex { nones: [ -1.0,  1.0] };
    let vertex4 = Vertex { nones: [  1.0,  1.0] };
    let shape = vec![vertex1, vertex2, vertex3, vertex4];

    let grid = {
        let mut grid = vec![];
        for i in 0..BOARD_SIZE_X {
            let coord = -1.0 + (i as f32) / (BOARD_SIZE_X as f32 / 2.0);
            grid.push( Vertex { nones: [coord, -1.0] } );
            grid.push( Vertex { nones: [coord,  1.0] } );
        }

        for i in 0..BOARD_SIZE_Y {
            let coord = -1.0 + (i as f32) / (BOARD_SIZE_Y as f32 / 2.0);
            grid.push( Vertex { nones: [-1.0, coord] } );
            grid.push( Vertex { nones: [ 1.0, coord] } );

        }

        let grid_vertex_buffer = glium::VertexBuffer::new(&display, &grid).unwrap();
        let grid_indices = glium::index::NoIndices(glium::index::PrimitiveType::LinesList);

        let grid_vertex_shader_src = r#"
            #version 140
            in vec2 nones;
            void main() {
                gl_Position = vec4(nones, 0.0, 1.0);
            }
            "#;

            let grid_fragment_shader_src = r#"
            #version 140
            out vec4 out_color;
            void main() {
                out_color = vec4(0.0, 0.0, 0.0, 1.0);
            }
            "#;

            let grid_program = glium::Program::from_source(&display, grid_vertex_shader_src, grid_fragment_shader_src, None).unwrap();

            RenderObject { 
                vertices: grid_vertex_buffer,
                indices:  grid_indices,
                // uniforms: glium::uniforms::EmptyUniforms,
                program:  grid_program
            }
    };

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);


    let entity_template = { 
        let vertex_shader_src = format!(r#"
            #version 140
            in vec2 nones;
            uniform vec2 position;
            void main() {{
                gl_Position = vec4(position + vec2(nones.x / {}, nones.y / {}), 0.0, 1.0);
            }}
            "#, BOARD_SIZE_X, BOARD_SIZE_Y);

            let fragment_shader_src = r#"
            #version 140
            out vec4 out_color;
            uniform vec3 color;
            void main() {
                out_color = vec4(color, 1.0);
            }
            "#;


            let program = glium::Program::from_source(&display, &vertex_shader_src, fragment_shader_src, None).unwrap();

            RenderObject { vertices: vertex_buffer,
                           indices: indices,
                           // uniforms: uniform! {color: (0.0, 0.0, 0.0), position: (0.0, 0.0) },
                           program: program
            }
        };

        RenderTarget { display: display,
                       grid: grid,
                       entity_template: entity_template,
                       mouse_pos: (0,0),
                    }
}

pub fn draw_update(world_board:&Board, render_target:&mut RenderTarget) -> bool {
    use glium::Surface;
    let mut frame = render_target.display.draw();
    frame.clear_color(1.0, 1.0, 1.0, 1.0);

    // frame.draw(&grid_vertex_buffer, &grid_indices, &grid_program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();
    draw(&render_target.grid, &glium::uniforms::EmptyUniforms, &mut frame);
    draw_board(&world_board, &render_target, &mut frame);
    frame.finish().unwrap();

    for ev in render_target.display.poll_events() {
        use glium::glutin::Event::*;
        match ev {
            Closed => return false,
            MouseMoved(x, y) => render_target.mouse_pos = (x as u32, y as u32),
            MouseInput(glium::glutin::ElementState::Pressed, glium::glutin::MouseButton::Left) => println!("{:?}", pixels_to_grid(render_target.mouse_pos.0, render_target.mouse_pos.1)),
            _ => ()
        }
    }

    return true;
}

fn draw_board (board:&Board, render_target:&RenderTarget, frame:&mut glium::Frame) {
    for column in board.entities.iter() {
        for entity in column.iter() {
            draw_entity(entity, render_target, frame);
        }
    }
}

fn draw_entity (entity:&Option<Entity>, render_target:&RenderTarget, frame:&mut glium::Frame) {
    if entity.is_some() {
        let entity = entity.unwrap();
        let uniforms = uniform! { color: entity.sprite(), position: entity.position().as_canvas() };
        draw(&render_target.entity_template, &uniforms, frame);
    }
}

fn draw<U> (r: &RenderObject, uniforms: &U, target:&mut glium::Frame)
    where U: glium::uniforms::Uniforms
{
    use glium::Surface;
    target.draw(&r.vertices, &r.indices, &r.program, uniforms, &Default::default()).unwrap();
}


impl Position {
    fn as_canvas(&self) -> (f32, f32) {
        self.map(|x, y| 
                        ((x as f32 / (BOARD_SIZE_X as f32) * 2.0 -1.0 - 1.0/(BOARD_SIZE_X as f32)),
                       - (y as f32 / (BOARD_SIZE_Y as f32) * 2.0 -1.0 - 1.0/(BOARD_SIZE_Y as f32))))
        // (  (self.0 as f32) /   
        //    *2.0 -1.0
        //    + 0.0/(CELL_SIZE_X as f32) ,
        //    -((self.1 as f32) / (BOARD_SIZE_Y as f32)
        //      *2.0 -1.0
        //      + 0.0/(CELL_SIZE_Y as f32) ) )
    }
}

fn pixels_to_grid(x:u32, y:u32) -> Position {
    Position::new(x / CELL_SIZE_X + 1, y/CELL_SIZE_Y + 1)
}
