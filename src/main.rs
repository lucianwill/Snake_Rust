extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use piston::Button::Keyboard;
use glutin_window::GlutinWindow;
use opengl_graphics::{ GlGraphics, OpenGL };
use rand::Rng;

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    gametype: GameType,
    entities: Vec<Entity>
}

enum GameType {
    Empty(usize, u32),
    Empty_Walled(usize, u32, f64, f64, f64, f64),
    Walled_Food(usize, u32, f64, f64, f64, f64, usize),
    Clutterer(usize, u32, f64, f64, f64, f64, usize)
}

enum Entity {
    Snake(Vec<Position>, Direction, Controller),
    Wall(Vec<Position>),
    Food(Position)
}

struct Position {
    x: f64,
    y: f64
}

struct ColoredRect {
    rect: graphics::types::Rectangle,
    color: [f32; 4]
}

enum Controller {
    Player(usize, u32, usize, u32)
}

enum Direction {
    Right,
    Left,
    Up,
    Down
}

impl Position {
    fn new(xpos: f64, ypos: f64) -> Position {
        Position {
            x: xpos,
            y: ypos
        }
    }

    fn collide(&self, other: &Entity) -> bool {
        match &other {
            Entity::Snake(pos_mem, dir, controller) => {
                for i in pos_mem {
                    if self.x == i.x && self.y == i.y {
                        return true;
                    }
                }
                return false;
            },
            Entity::Wall(pos_mem) => {
                for i in pos_mem {
                    if self.x == i.x && self.y == i.y {
                        return true;
                    }
                }
                return false;
            },
            Entity::Food(pos) => {
                if self.x == pos.x && self.y == pos.y {
                    return true;
                }
                return false;
            }
        }
    }

    fn extend(&self, target: &Position) -> Vec<Position> {
        let mut v: Vec<Position> = Vec::new();

        let sx = self.x as i64;
        let tx = target.x as i64;
        let sy = self.y as i64;
        let ty = target.y as i64;

        if self.x <= target.x {
            for i in 0..tx - sx {
                v.push(Position::new(self.x + (i as f64), self.y));
            }
        } else {
            for i in 0..sx - tx {
                v.push(Position::new(self.x - (i as f64), self.y));
            }
        }

        if self.y <= target.y {
            for i in 0..ty - sy {
                v.push(Position::new(target.x, self.y + (i as f64)));
            }
        } else {
            for i in 0..sy - ty {
                v.push(Position::new(target.x, self.y - (i as f64)));
            }
        }
        v
    }
}

impl Entity {
    fn new_Snake(p: Position, d: Direction) -> Entity {
        Entity::Snake(vec![p], d, Controller::Player(5, 0, 5, 5))
    }

    fn custom_Snake(p: Position, d: Direction, c: Controller) -> Entity {
        Entity::Snake(vec![p], d, c)
    }

    fn new_Wall(start: Position, end: Position) -> Entity {
        Entity::Wall(start.extend(&end))
    }

    fn turn(&mut self, d: Direction) {
        if let Entity::Snake(pos_mem, ref mut dir, controller) = self {
            let valid: bool = true;

            match d {
                Direction::Right => {
                    if !(pos_mem[0].x == pos_mem[1].x - 1.0 && pos_mem[0].y == pos_mem[1].y) {
                        *dir = d;
                    }
                },
                Direction::Left => {
                    if !(pos_mem[0].x == pos_mem[1].x + 1.0 && pos_mem[0].y == pos_mem[1].y) {
                        *dir = d;
                    }
                },
                Direction::Up => {
                    if !(pos_mem[0].x == pos_mem[1].x && pos_mem[0].y == pos_mem[1].y + 1.0) {
                        *dir = d;
                    }
                },
                Direction::Down => {
                    if !(pos_mem[0].x == pos_mem[1].x && pos_mem[0].y == pos_mem[1].y - 1.0) {
                        *dir = d;
                    }
                }
            }
        }
    }

    fn kill(self) -> Entity {
        if let Entity::Wall(pos_mem) = &self {
            return self;
        }

        if let Entity::Snake(pos_mem, dir, controller) = self {
            return Entity::Wall(pos_mem);
        }

        Entity::Wall(vec![Position::new(0.0, 0.0)])
    }

    fn step(mut entities: Vec<Entity>) -> Vec<Entity> {
        let mut snake_indices: Vec<usize> = Vec::new();
        let mut obs_indices:   Vec<usize> = Vec::new();
        let mut wall_indices:  Vec<usize> = Vec::new();
        let mut food_indices:  Vec<usize> = Vec::new();
        let mut drop_indices:  Vec<usize> = Vec::new();
        let mut return_vector: Vec<Entity> = Vec::new();

        for (i, ent) in entities.iter().enumerate() {
            match &ent {
                Entity::Snake(a,b,c) => {
                    snake_indices.push(i);
                    obs_indices.push(i);
                },
                Entity::Wall(a) => {
                    wall_indices.push(i);
                    obs_indices.push(i);
                },
                Entity::Food(a) => food_indices.push(i),
            }
        }

        for i in &snake_indices {
            let mut next = Position::new(0.0, 0.0);
            let mut can_move = false;

            if let Entity::Snake(pos_mem, dir, ref mut controller) = &mut entities[*i] {
                next = match dir {
                    Direction::Right => Position::new(pos_mem[0].x + 1.0, pos_mem[0].y),
                    Direction::Left =>  Position::new(pos_mem[0].x - 1.0, pos_mem[0].y),
                    Direction::Up =>    Position::new(pos_mem[0].x, pos_mem[0].y - 1.0),
                    Direction::Down =>  Position::new(pos_mem[0].x, pos_mem[0].y + 1.0),
                };

                match controller {
                    Controller::Player(target_len, ref mut count, grow_len, speed) => {
                        *count += 1;
                        if *count == *speed {
                            *count = 0;
                        }
                        can_move = *count % *speed == 0;
                    },
                };
            }

            if can_move {
                for j in &obs_indices {
                    if next.collide(&entities[*j]) {
                        let mut temp = entities.remove(*i);
                        temp = temp.kill();
                        entities.insert(*i, temp);
                    }
                }

                for j in &food_indices {
                    if next.collide(&entities[*j]) {
                        if let Entity::Snake(a,b, ref mut control) = &mut entities[*i] {
                            match control {
                                Controller::Player(ref mut target_len, count, grow_len, speed) => {
                                    *target_len += *grow_len;
                                },
                            }
                        }
                        drop_indices.push(*j);
                    }
                }

                if let Entity::Snake(ref mut pos_mem, b, control) = &mut entities[*i] {
                    match &control {
                        Controller::Player(target_len, count, grow_len, speed) => {
                            if *target_len == pos_mem.len() {
                                pos_mem.pop();
                            }
                            pos_mem.insert(0, next);
                        }
                    }
                }
            }
        }

        drop_indices.sort();

        for i in drop_indices.iter().rev() {
            entities.remove(*i);
        }

        entities
    }

    fn gen_squares(&self) -> Vec<ColoredRect> {
        use graphics::*;

        let mut squares: Vec<ColoredRect> = Vec::new();

        const RED:  [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
        const GRAY: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

        match &self {
            Entity::Snake(pos_mem, dir, controller) => {
                for i in pos_mem {
                    squares.push(ColoredRect {
                        rect: rectangle::square(i.x * 10.0, i.y * 10.0, 10.0),
                        color: BLUE
                    });
                }
                return squares;
            },
            Entity::Wall(pos_mem) => {
                for i in pos_mem {
                    squares.push(ColoredRect {
                        rect: rectangle::square(i.x * 10.0, i.y * 10.0, 10.0),
                        color: GRAY
                    });
                }
                return squares;
            },
            Entity::Food(pos) => {
                squares.push(ColoredRect {
                    rect: rectangle::square(pos.x * 10.0, pos.y * 10.0, 10.0),
                    color: RED
                });
                return squares;
            },
        }
    }
}

impl App {
    fn new_Game(g: GlGraphics, gmtyp: GameType) -> App {
        match gmtyp {
            GameType::Empty(grow_len, speed) => App {
                gl: g,
                gametype: gmtyp,
                entities: vec![Entity::custom_Snake(Position::new(0.0, 0.0), Direction::Right, Controller::Player(5, 0, grow_len, speed))]
            },
            GameType::Empty_Walled(grow_len, speed, x1, y1, x2, y2) => App {
                gl: g,
                gametype: gmtyp,
                entities: vec![Entity::custom_Snake(Position::new(95.0, 53.0), Direction::Right, Controller::Player(5, 0, grow_len, speed)),
                                                 Entity::new_Wall(Position::new(x1, y1), Position::new(x2, y2)),
                                                 Entity::new_Wall(Position::new(x2, y2), Position::new(x1, y1))]
            },
            GameType::Walled_Food(grow_len, speed, x1, y1, x2, y2, food_count) => {
                let mut temp = App {
                    gl: g,
                    gametype: gmtyp,
                    entities: vec![Entity::custom_Snake(Position::new(95.0, 53.0), Direction::Right, Controller::Player(5, 0, grow_len, speed)),
                                                     Entity::new_Wall(Position::new(x1, y1), Position::new(x2, y2)),
                                                     Entity::new_Wall(Position::new(x2, y2), Position::new(x1, y1))]
                };
                let mut valid = true;

                while temp.entities.len() - 3 < food_count {
                    valid = true;

                    let mut pos = Position::new(((rand::random::<f64>() * (x2 - x1 - 1.0)) + 1.0).trunc(),
                                                ((rand::random::<f64>() * (y2 - y1 - 1.0)) + 1.0).trunc());

                    if pos.collide(&temp.entities[0]) {
                        valid = false;
                    } else {
                        for i in 3..temp.entities.len() {
                            if pos.collide(&temp.entities[i]) {
                                valid = false;
                            }
                        }
                    }

                    if valid {
                        temp.entities.push(Entity::Food(pos));
                    }
                }
                temp
            },
            GameType::Clutterer(grow_len, speed, x1, y1, x2, y2, food_count) => {
                let mut temp = App {
                    gl: g,
                    gametype: gmtyp,
                    entities: vec![Entity::custom_Snake(Position::new(95.0, 53.0), Direction::Right, Controller::Player(5, 0, grow_len, speed)),
                                                     Entity::new_Wall(Position::new(x1, y1), Position::new(x2, y2)),
                                                     Entity::new_Wall(Position::new(x2, y2), Position::new(x1, y1))]
                };
                let mut valid = true;

                while temp.entities.len() - 3 < food_count {
                    valid = true;

                    let mut pos = Position::new(((rand::random::<f64>() * (x2 - x1 - 1.0)) + 1.0).trunc(),
                                                ((rand::random::<f64>() * (y2 - y1 - 1.0)) + 1.0).trunc());

                    if pos.collide(&temp.entities[0]) {
                        valid = false;
                    } else {
                        for i in 3..temp.entities.len() {
                            if pos.collide(&temp.entities[i]) {
                                valid = false;
                            }
                        }
                    }

                    if valid {
                        temp.entities.push(Entity::Food(pos));
                    }
                }
                temp
            },
        }
    }

    fn reset(&mut self) {
        match &mut self.gametype {
            GameType::Empty(grow_len, speed) => self.entities = vec![Entity::custom_Snake(Position::new(95.0, 53.0), Direction::Right, Controller::Player(5, 0, *grow_len, *speed))],
            GameType::Empty_Walled(grow_len, speed, x1, y1, x2, y2) =>
                self.entities[0] = Entity::custom_Snake(Position::new(95.0, 53.0), Direction::Right, Controller::Player(5, 0, *grow_len, *speed)),
            GameType::Walled_Food(grow_len, speed, x1, y1, x2, y2, food_count) => {
                self.entities[0] = Entity::custom_Snake(Position::new(95.0, 53.0), Direction::Right, Controller::Player(5, 0, *grow_len, *speed));
                self.entities.truncate(3);
                let mut valid = true;

                while self.entities.len() - 3 < *food_count {
                    valid = true;

                    let mut pos = Position::new(((rand::random::<f64>() * (*x2 - *x1 - 1.0)) + 1.0).trunc(),
                                                ((rand::random::<f64>() * (*y2 - *y1 - 1.0)) + 1.0).trunc());

                    if pos.collide(&self.entities[0]) {
                        valid = false;
                    } else {
                        for i in 3..self.entities.len() {
                            if pos.collide(&self.entities[i]) {
                                valid = false;
                            }
                        }
                    }

                    if valid {
                        self.entities.push(Entity::Food(pos));
                    }
                }
            },
            GameType::Clutterer(grow_len, speed, x1, y1, x2, y2, food_count) => {
                self.entities[0] = Entity::custom_Snake(Position::new(95.0, 53.0), Direction::Right, Controller::Player(5, 0, *grow_len, *speed));
                //XXX Truncation is not correct here
                self.entities.truncate(3);
                let mut valid = true;

                while self.entities.len() - 3 < *food_count {
                    valid = true;

                    let mut pos = Position::new(((rand::random::<f64>() * (*x2 - *x1 - 1.0)) + 1.0).trunc(),
                                                ((rand::random::<f64>() * (*y2 - *y1 - 1.0)) + 1.0).trunc());

                    if pos.collide(&self.entities[0]) {
                        valid = false;
                    } else {
                        for i in 3..self.entities.len() {
                            if pos.collide(&self.entities[i]) {
                                valid = false;
                            }
                        }
                    }

                    if valid {
                        self.entities.push(Entity::Food(pos));
                    }
                }
            },
        }
    }

    fn spawn_food(&mut self) {
        if let GameType::Walled_Food(grow_len, speed, x1, y1, x2, y2, food_count) = &mut self.gametype {
            let mut valid = true;

            while self.entities.len() - 3 < *food_count {
                valid = true;

                let mut pos = Position::new(((rand::random::<f64>() * (*x2 - *x1 - 1.0)) + 1.0).trunc(),
                                            ((rand::random::<f64>() * (*y2 - *y1 - 1.0)) + 1.0).trunc());

                if pos.collide(&self.entities[0]) {
                    valid = false;
                } else {
                    for i in 3..self.entities.len() {
                        if pos.collide(&self.entities[i]) {
                            valid = false;
                        }
                    }
                }

                if valid {
                    self.entities.push(Entity::Food(pos));
                }
            }
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BACK: [f32; 4] = [0.4, 0.4, 0.0, 1.0];
        const RED:  [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
        const GRAY: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

        let mut squares: Vec<ColoredRect> = Vec::new();

        for i in &self.entities {
            squares.append(&mut i.gen_squares());
        }

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BACK, gl);

            let transform = c.transform;

            for i in squares {
                rectangle(i.color, i.rect, transform, gl);
            }
        });
    }

    fn update(mut self, args: &UpdateArgs) -> App {
        self.entities = Entity::step(self.entities);
        self.spawn_food();
        return self;
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    let mut events = Events::new(EventSettings::new());

    // Create an Glutin window.
    let mut window: GlutinWindow = WindowSettings::new(
            "spinning-square",
            (1920, 1080)
        )
        .graphics_api(opengl)
        .exit_on_esc(true)
        .fullscreen(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    //let mut app = App::new_Game(GlGraphics::new(opengl), GameType::Empty(5, 5));
    //let mut app = App::new_Game(GlGraphics::new(opengl), GameType::Empty_Walled(5, 5, 0.0, 0.0, 191.0, 107.0));
    let mut app = App::new_Game(GlGraphics::new(opengl), GameType::Walled_Food(200, 2, 0.0, 0.0, 191.0, 107.0, 5));

    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app = app.update(&u);
        }

        if let Some(b) = e.button_args() {
            if let Keyboard(Key) = b.button {
                match Key {
                    Key::Right => app.entities[0].turn(Direction::Right),
                    Key::Left => app.entities[0].turn(Direction::Left),
                    Key::Up => app.entities[0].turn(Direction::Up),
                    Key::Down => app.entities[0].turn(Direction::Down),
                    Key::R => app.reset(),
                    _ => (),
                }
            }
        }
    }
}
