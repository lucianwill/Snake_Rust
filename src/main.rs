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

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64,   // Rotation for the square.
    size: f64,
    entities: Vec<Entity>
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
    Player(usize, u32)
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
    fn new_Snake(p: Position, d: Direction, c: Controller) -> Entity {
        Entity::Snake(vec![p], d, Controller::Player(5, 0))
    }

    fn new_Wall(start: Position, end: Position) -> Entity {
        Entity::Wall(start.extend(&end))
    }

    fn new_Food(p: Position) -> Entity {
        Entity::Food(p)
    }

    fn turn(&mut self, d: Direction) {
        if let Entity::Snake(pos_mem, ref mut dir, controller) = self {
            match d {
                Direction::Right => {
                    if let Direction::Left = dir { } else {
                        *dir = d;
                    }
                },
                Direction::Left => {
                    if let Direction::Right = dir { } else {
                        *dir = d;
                    }
                },
                Direction::Up => {
                    if let Direction::Down = dir { } else {
                        *dir = d;
                    }
                },
                Direction::Down => {
                    if let Direction::Up = dir { } else {
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

            if let Entity::Snake(pos_mem, dir, ref mut controller) = &mut entities[*i] {
                next = match dir {
                    Direction::Right => Position::new(pos_mem[0].x + 1.0, pos_mem[0].y),
                    Direction::Left =>  Position::new(pos_mem[0].x - 1.0, pos_mem[0].y),
                    Direction::Up =>    Position::new(pos_mem[0].x, pos_mem[0].y - 1.0),
                    Direction::Down =>  Position::new(pos_mem[0].x, pos_mem[0].y + 1.0),
                };

                match controller {
                    Controller::Player(target_len, ref mut count) => {
                        *count += 1;
                        if *count == 840 {
                            *count = 0;
                        }
                    },
                };
            }

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
                            Controller::Player(ref mut target_len, count) => {
                                *target_len += 5;
                            },
                        }
                    }
                    drop_indices.push(*j);
                }
            }

            if let Entity::Snake(ref mut pos_mem, b, control) = &mut entities[*i] {
                match &control {
                    Controller::Player(target_len, count) => {
                        if count % 5 == 0 {
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
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BACK: [f32; 4] = [0.8, 0.2, 0.0, 1.0];
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
    let mut app = App {
        gl: GlGraphics::new(opengl),
        rotation: 0.0,
        size: 200.0,
        entities: vec![Entity::new_Snake(Position::new(5.0, 5.0), Direction::Right, Controller::Player(5, 0))]
    };

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
                    _ => (),
                }
            }
        }
    }
}
