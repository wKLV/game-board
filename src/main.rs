#[macro_use]
extern crate glium;

mod io;

mod game {
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
#[derive(Copy, Clone, Hash, Debug)]
    pub struct Position(u32, u32);

    impl Position {
        pub fn map<T, F:FnOnce(u32, u32)->T>(&self, f:F)->T {
            f(self.0, self.1)
        }
        pub fn new(x:u32, y:u32) -> Position {
            assert!(x >= 1 && y >= 1);
            Position(x, y)
        }
    }

#[derive(Copy, Clone, Debug)]
    pub enum EntityType {
        Chara,
        Monstar,
    }

#[derive(Copy, Clone, Debug)]
    pub struct Entity {
        entity_type: EntityType,
        position: Position,
    }

    impl Entity {
        pub fn sprite(&self) -> (f32, f32, f32) {
            use self::EntityType::*;
            match self.entity_type {
                Chara => (0.5, 0.75, 0.25),
                Monstar => (0.75, 0.5, 0.25)
            }
        }

        pub fn position(&self) -> Position {
            self.position
        }
    }

    pub const BOARD_SIZE_X:u32 = 8;
    pub const BOARD_SIZE_Y:u32 = 5;

#[derive(Copy, Clone, Debug)]
    pub struct Board {
        pub entities : [[Option<Entity>;BOARD_SIZE_Y as usize];BOARD_SIZE_X as usize], 
    }

    impl Board {
        pub fn new () -> Board {
            Board { entities: [[ None; BOARD_SIZE_Y as usize ] ; BOARD_SIZE_X as usize ] }
        }

        pub fn try_add_entity(&mut self, pos:Position, t:EntityType) -> Option<Entity> {
            let entity_created = Some( Entity {position:pos, entity_type:t} ); 
            if self[pos].is_none() {
                self[pos] = entity_created;;
                return entity_created;
            }
            return None;
        }
    }

    use std::ops::{Index, IndexMut};
    impl Index<Position> for Board {
        type Output = Option<Entity>;

        fn index(&self, index:Position) -> &Option<Entity> {
            &self.entities[index.0 as usize -1][index.1 as usize -1]
        }
    }
    impl IndexMut<Position> for Board {
        fn index_mut(&mut self, index:Position) -> &mut Option<Entity> {
            &mut self.entities[index.0 as usize -1][index.1 as usize -1]
        }
    }
}

// CODE

fn main() {
    use game::EntityType::*;
    use game::*;

    let mut world_board = Board::new();
    world_board.try_add_entity(Position::new(1,1), Chara);
    world_board.try_add_entity(Position::new(2,2), Monstar);

    let mut display = io::init();

    use io::Input::*;
    loop {
        if let Some(result) = io::draw_update(&world_board, &mut display) {
            match result {
                Quit => return,
                TileClick(pos) => println!("{:?}", pos),
                _ =>  println!("{:?}", result),
            }
        }
    }
}
