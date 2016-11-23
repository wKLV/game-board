use glium;
use game::*;
use glium::Surface;
use glium::Rect;
use image;

#[derive(Copy, Clone, Debug)]
pub enum Input {
    Quit,
    TileClick(Position),
    Action1,
//    Action2,
//    ActionAOn(Position),
//    ActionBOn(Position),
}


#[derive(Copy, Clone, Debug)]
struct Vertex {
    pos: [f32; 2],
    tex: [f32; 2],
}
implement_vertex!(Vertex, pos, tex);

struct RenderGrid {
    vertices: glium::VertexBuffer<Vertex>,
    indices:  glium::index::NoIndices,
    program:  glium::Program,
}

impl RenderGrid {
    fn draw<S:Surface>(&self, target:&mut S) {
        target.draw(&self.vertices, &self.indices, &self.program, &glium::uniforms::EmptyUniforms, 
                    &glium::draw_parameters::DrawParameters{ blend: glium::Blend::alpha_blending(), ..Default::default()}).unwrap();
    }
}

struct RenderEntity {
    vertices: glium::VertexBuffer<Vertex>,
    indices:  glium::index::NoIndices,
    program: glium::Program,
    texture: glium::texture::texture2d::Texture2d,
    texture_size: (u32, u32),
}

impl RenderEntity {
    fn draw<S:Surface>(&self, target:&mut S, entity:&Entity) {
        let (sprite_begin, sprite_size) = entity.sprite().tex_coords_scaled(self.texture_size.0 as f32, self.texture_size.1 as f32);
        let uniforms = uniform!{ sprite_begin: sprite_begin, sprite_size:sprite_size, 
            tex: glium::uniforms::Sampler::new(&self.texture).magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest), 
            position: entity.position().as_canvas()}; 
        target.draw(&self.vertices, &self.indices, &self.program, &uniforms, 
                    &glium::draw_parameters::DrawParameters{ blend: glium::Blend::alpha_blending(), ..Default::default()}).unwrap();
    }
}

pub struct UIState {
    display: glium::backend::glutin_backend::GlutinFacade,
    grid: RenderGrid,
    entity_template: RenderEntity,
    mouse_pos: (u32, u32)
}

impl Sprite {
    // this would change if the sprit would change!
    pub fn tex_coords(&self) -> Rect  {
        match *self {
            Sprite::Chara   => Rect{ left:0, bottom:0,   width:16, height:16},
            Sprite::Monstar => Rect{ left:0, bottom:136, width:16, height:16},
        }
    }

    pub fn tex_coords_scaled(&self, x:f32, y:f32) -> ((f32, f32), (f32, f32)) {
        let rect = self.tex_coords();
        ((rect.left as f32 / x,  rect.bottom as f32 / y),
         (rect.width as f32 / x, rect.height as f32 / y))
    }
}

const SCREEN_PIXELS_X:u32 = 800;
const SCREEN_PIXELS_Y:u32 = 500;

const CELL_SIZE_X:u32 = (SCREEN_PIXELS_X / BOARD_SIZE_X);
const CELL_SIZE_Y:u32 = (SCREEN_PIXELS_Y / BOARD_SIZE_Y);

pub fn init() -> UIState {
    use glium::DisplayBuild;

    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(SCREEN_PIXELS_X as u32, SCREEN_PIXELS_Y as u32)
        .with_min_dimensions(SCREEN_PIXELS_X as u32, SCREEN_PIXELS_Y as u32)
        .with_max_dimensions(SCREEN_PIXELS_X as u32, SCREEN_PIXELS_Y as u32)
        .with_title(format!("tt"))
        .build_glium()
        .unwrap();

    use std::io::Cursor;
    let image = image::load(Cursor::new(&include_bytes!("../res/roguelikeChar.png")[..]),
    image::PNG).unwrap().to_rgba();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
    let texture = glium::texture::Texture2d::new(&display, image).unwrap();

    let grid = {
        let mut grid = vec![];
        for i in 0..BOARD_SIZE_X {
            let coord = -1.0 + (i as f32) / (BOARD_SIZE_X as f32 / 2.0);
            grid.push( Vertex { pos: [coord, -1.0], tex:[0.0, 0.0] } );
            grid.push( Vertex { pos: [coord,  1.0], tex:[0.0, 0.0] } );
        }

        for i in 0..BOARD_SIZE_Y {
            let coord = -1.0 + (i as f32) / (BOARD_SIZE_Y as f32 / 2.0);
            grid.push( Vertex { pos: [-1.0, coord], tex:[0.0, 0.0] } );
            grid.push( Vertex { pos: [ 1.0, coord], tex:[0.0, 0.0] } );

        }

        let grid_vertex_buffer = glium::VertexBuffer::new(&display, &grid).unwrap();
        let grid_indices = glium::index::NoIndices(glium::index::PrimitiveType::LinesList);

        let grid_vertex_shader_src = r#"
            #version 140
            in vec2 pos;
            void main() {
                gl_Position = vec4(pos, 0.0, 1.0);
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

            RenderGrid { 
                vertices: grid_vertex_buffer,
                indices:  grid_indices,
                program:  grid_program
            }
    };


    let entity_template = { 
        let vertex1 = Vertex { pos: [ -1.0, -1.0], tex:[0.0, 0.0] };
        let vertex2 = Vertex { pos: [  1.0, -1.0], tex:[1.0, 0.0] };
        let vertex3 = Vertex { pos: [ -1.0,  1.0], tex:[0.0, 1.0] };
        let vertex4 = Vertex { pos: [  1.0,  1.0], tex:[1.0, 1.0] };
        let shape = vec![vertex1, vertex2, vertex3, vertex4];

        let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

        let vertex_shader_src = format!(r#"
            #version 140
            in vec2 pos;
            in vec2 tex;
            out vec2 tex_coords;
            uniform vec2 position;
            void main() {{
                gl_Position = vec4(position + vec2(pos.x / {}, pos.y / {}), 0.0, 1.0);
                tex_coords = tex;
            }}
            "#, BOARD_SIZE_X, BOARD_SIZE_Y);

            let fragment_shader_src = r#"
            #version 140
            in vec2 tex_coords;
            out vec4 out_color;

            uniform sampler2D tex;
            uniform vec2 sprite_begin;
            uniform vec2 sprite_size;
            void main() {{
                out_color = texture(tex, sprite_size*tex_coords + sprite_begin);
            }}
            "#;


            let program = glium::Program::from_source(&display, &vertex_shader_src, fragment_shader_src, None).unwrap();

            RenderEntity { vertices: vertex_buffer,
                           indices: indices,
                           program: program,
                           texture: texture,
                           texture_size: image_dimensions,
            }
        };

        UIState { display: display,
                       grid: grid,
                       entity_template: entity_template,
                       mouse_pos: (0,0),
                    }
}

pub fn draw_update(world_board:&Board, ui_state:&mut UIState) -> Option<Input> {
    use glium::Surface;
    let mut frame = ui_state.display.draw();
    frame.clear_color(1.0, 1.0, 1.0, 1.0);

    ui_state.grid.draw(&mut frame);
    draw_board(&world_board, &ui_state, &mut frame);
    frame.finish().unwrap();
    
    let mut result:Option<Input> = None;
    use io::Input::*;
    use glium::glutin::ElementState::Pressed;
    for ev in ui_state.display.poll_events() {
        use glium::glutin::Event::*;
        match ev {
            Closed => result=Some(Quit),
            MouseMoved(x, y) => ui_state.mouse_pos = (x as u32, y as u32),
            MouseInput(Pressed, glium::glutin::MouseButton::Left) => result = Some(TileClick(pixels_to_grid(ui_state.mouse_pos.0, ui_state.mouse_pos.1))),
            KeyboardInput(Pressed, _, Some(glium::glutin::VirtualKeyCode::X)) => result = Some(Action1),
            _ => ()
        }
    }
    return result;
}

fn draw_board (board:&Board, ui_state:&UIState, frame:&mut glium::Frame) {
    for column in board.entities.iter() {
        for entity in column.iter() {
            if entity.is_some() {
                ui_state.entity_template.draw(frame, &entity.unwrap());
            }
        }
    }
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
