#[macro_use]
extern crate glium;

// HEADER

// #[derive(Copy, Clone, Debug)]
// enum Dir {
//     Up,
//     Left,
//     Right,
//     Down,
// }

// #[derive(Copy, Clone, Debug)]
// enum Action {
//     Move(Dir),
//     Attack,
// }

#[derive(Copy, Clone, Debug)]
enum Entity {
    Chara,
    Monstar,
}

fn sprite(e:Entity) -> (f32, f32, f32) {
    use Entity::*;
    match e {
        Chara => (0.5, 0.75, 0.25),
        Monstar => (0.75, 0.5, 0.25)
    }
}

const BOARD_SIZE_X:usize = 8;
const BOARD_SIZE_Y:usize = 8;

const SCREEN_PIXELS_X:usize = 800;
const SCREEN_PIXELS_Y:usize = 800;

const GRID_SIZE_X:usize = (SCREEN_PIXELS_X / BOARD_SIZE_X);
const GRID_SIZE_Y:usize = (SCREEN_PIXELS_Y / BOARD_SIZE_Y);

struct Board {
    entities : [[Option<Entity>;BOARD_SIZE_Y];BOARD_SIZE_X], 
}

impl Board {
    fn new () -> Board {
        Board { entities: [[ None; BOARD_SIZE_Y ] ; BOARD_SIZE_X ] }
    }
}


#[derive(Copy, Clone, Debug)]
struct Vertex {
    nones: [f32; 2],
}
implement_vertex!(Vertex, nones);

// CODE

fn main() {
    use glium::{DisplayBuild, Surface};
    use Entity::*;

    let mut world_board = Board::new();

    world_board.entities[0][0] = Some(Chara);
    world_board.entities[1][1] = Some(Monstar);

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

    let mut grid = vec![];
    // TODO: BOARD_SIZE_X =/= BOARD_SIZE_Y
    for i in 0..BOARD_SIZE_X {
        let coord = -1.0 + (i as f32) / (8.0 / 2.0);
        grid.push( Vertex { nones: [coord, -1.0] } );
        grid.push( Vertex { nones: [coord,  1.0] } );
        grid.push( Vertex { nones: [-1.0, coord] } );
        grid.push( Vertex { nones: [ 1.0, coord] } );
        
    }

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

    let grid_vertex_buffer = glium::VertexBuffer::new(&display, &grid).unwrap();
    let grid_indices = glium::index::NoIndices(glium::index::PrimitiveType::LinesList);

    let vertex_shader_src = r#"
        #version 140
        in vec2 nones;
        uniform vec2 position;
        void main() {
            gl_Position = vec4(position + nones*1.0/8.0, 0.0, 1.0);
        }
        "#;

    let fragment_shader_src = r#"
        #version 140
        out vec4 out_color;
        uniform vec3 color;
        void main() {
            out_color = vec4(color, 1.0);
        }
        "#;

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

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();
    let grid_program = glium::Program::from_source(&display, grid_vertex_shader_src, grid_fragment_shader_src, None).unwrap();

    let draw = |board:&Board, target:&mut glium::Frame| {
        for (x, es) in board.entities.iter().enumerate() {
            for (y, e) in es.iter().enumerate() {
                if let Some(entity) = *e {
                    let color = sprite(entity);
                    let uniforms = uniform! { color: color, position:((x as f32)*0.25f32 -1.0 +1.0/8.0, -(y as f32)*0.25f32 +1.0 -1.0/8.0) };
                    target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();

                }
            }
        }
    };

    let mut mouse_pos: Option<(i32, i32)> = None;

    fn pixels_to_grid(pixels:Option<(i32, i32)>) -> Option<(usize, usize)> {
        // println!("{:?}", (GRID_SIZE_X, GRID_SIZE_Y));
        match pixels {
            None => None,
            Some((x, y)) => Some(
                ((x as usize) / GRID_SIZE_X,
                 (y as usize) / GRID_SIZE_Y))
        }
    }

    loop {
        let mut target = display.draw();
        target.clear_color(1.0, 1.0, 1.0, 1.0);
        //target.draw(&vertex_buffer, &indices, &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();
        target.draw(&grid_vertex_buffer, &grid_indices, &grid_program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();
        draw(&world_board, &mut target);
        target.finish().unwrap();
        for ev in display.poll_events() {
            use glium::glutin::Event::*;
            match ev {
                Closed => return,
                MouseMoved(x, y) => mouse_pos = Some((x,y)),
                MouseInput(glium::glutin::ElementState::Pressed, glium::glutin::MouseButton::Left) => println!("{:?}", pixels_to_grid(mouse_pos)),
                _ => ()
            }
        }
    }

}
