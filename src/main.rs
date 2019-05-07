extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

use std::collections::LinkedList;
use std::iter::FromIterator;
use rand::Rng;
use std::{thread, time};

static BACKGROUND_COLOR: [f32; 4] = [0.56, 0.93, 0.56, 1.0];
static SNAKE_HEAD_COLOR: [f32; 4] = [1.0, 0.27, 0.0, 1.0];
static SNAKE_BODY_COLOR: [f32; 4] = [1.0, 0.50, 0.31, 1.0];
static FOOD_COLOR: [f32; 4] = [1.0, 1.0, 0.88, 1.0];

#[derive(Clone, PartialEq)]
enum GameStatus {
    Running,
    Pause,
    TitleScreen,
    End,
}

#[derive(Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right
}

struct Game {
    width: u32,
    pixels_per_case: u32,
    gl: GlGraphics,
    score: u32,
    snake: Snake,
    food: Food,
    status: GameStatus
}

impl Game {
    fn render(&mut self, arg: &RenderArgs) {
        use graphics;

        self.gl.draw(arg.viewport(), |_c, gl| {
            graphics::clear(BACKGROUND_COLOR, gl);
        });

        self.food.render(&mut self.gl, arg, self.pixels_per_case);
        self.snake.render(&mut self.gl, arg, self.pixels_per_case);
    }

    fn end(&mut self) {
        println!("End of game.");
        println!("Your score : {}", self.score);
    }

    pub fn update(&mut self) -> bool {
        if self.snake.will_hit_wall(self.width, self.pixels_per_case) == true {
            self.end();
            return false
        }

        if self.snake.bites_itself() == true {
            self.end();
            return false
        }

        if self.snake_eats_food() == true {
            self.score += 1;
            println!("New score : {}", self.score);
            self.snake.grow(self.width, self.pixels_per_case);
            self.food.update(self.width, self.pixels_per_case, &self.snake);
        }

        self.snake.update();

        return true
    }

    fn snake_eats_food(&mut self) -> bool {
        if let Some(snake_head) = self.snake.body.front() {
            if snake_head.0 == self.food.pos_x && snake_head.1 == self.food.pos_y {
                return true
            }
        }
        return false
    }

    pub fn pressed(&mut self, btn: &Button) {
        let last_direction = self.snake.direction.clone();

        self.snake.direction = match btn {
            &Button::Keyboard(Key::Up) if last_direction != Direction::Down => Direction::Up,
            &Button::Keyboard(Key::W) if last_direction != Direction::Down => Direction::Up,
            &Button::Keyboard(Key::Down) if last_direction != Direction::Up => Direction::Down,
            &Button::Keyboard(Key::S) if last_direction != Direction::Up => Direction::Down,
            &Button::Keyboard(Key::Left) if last_direction != Direction::Right => Direction::Left,
            &Button::Keyboard(Key::A) if last_direction != Direction::Right => Direction::Left,
            &Button::Keyboard(Key::Right) if last_direction != Direction::Left => Direction::Right,
            &Button::Keyboard(Key::D) if last_direction != Direction::Left => Direction::Right,
            _ => last_direction,
        };
    }
}

struct Snake {
    body: LinkedList<(u32, u32)>,
    direction: Direction
}

impl Snake {
    pub fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs, pixels: u32) {
        use graphics;

        let mut squares: Vec<graphics::types::Rectangle> = self.body
            .iter()
            .map(|&(x, y)| {
                graphics::rectangle::square(
                    (x * pixels) as f64,
                    (y * pixels) as f64,
                    pixels as f64)
            })
            .collect();

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            if let Some(&first) = squares.first() {
                squares.remove(0);
                squares.into_iter().for_each(|square| graphics::rectangle(SNAKE_BODY_COLOR, square, transform, gl));
                graphics::rectangle(SNAKE_HEAD_COLOR, first, transform, gl);
            };
        });
    }

    pub fn bites_itself(&mut self) -> bool {
        if let Some(snake_head) = self.body.front() {
            if collision_count(snake_head, &self.body) != 1 {
                return true
            }
        }
        return false
    }

    pub fn will_hit_wall(&mut self, width: u32, pixels: u32) -> bool {
        if let Some(snake_head) = self.body.front() {
            match self.direction {
                Direction::Up if snake_head.1 == 0 => return true,
                Direction::Down if snake_head.1 >= (width / pixels - 1) => return true,
                Direction::Left if snake_head.0 == 0 => return true,
                Direction::Right if snake_head.0 >= (width / pixels - 1) => return true,
                _ => return false
            }
        }
        return false
    }

    pub fn update(&mut self) {
        let mut new_head = (*self.body.front().expect("No front of snake found.")).clone();

        match self.direction {
            Direction::Left => new_head.0 -= 1,
            Direction::Right => new_head.0 += 1,
            Direction::Down => new_head.1 += 1,
            Direction::Up => new_head.1 -= 1,
        }

        self.body.push_front(new_head);
        self.body.pop_back().unwrap();
    }

    pub fn grow(&mut self, width: u32, pixels: u32) {
        self.body.push_back((width / pixels, width / pixels));
    }
}

struct Food {
    pos_x: u32,
    pos_y: u32
}

impl Food {
    pub fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs, pixels: u32) {
        use graphics;

        let square = graphics::rectangle::square(
            (self.pos_x * pixels) as f64,
            (self.pos_y * pixels) as f64,
            pixels as f64);

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            graphics::rectangle(FOOD_COLOR, square, transform, gl)
        });
    }

    pub fn update(&mut self, width: u32, pixels_per_case: u32, snake: &Snake) {
        let mut rand_x = rand::thread_rng().gen_range(0, width / pixels_per_case - 1);
        let mut rand_y = rand::thread_rng().gen_range(0, width / pixels_per_case - 1);
        while collision_count(&(rand_x, rand_y), &snake.body) != 0 {
            rand_x = rand::thread_rng().gen_range(0, width / pixels_per_case - 1);
            rand_y = rand::thread_rng().gen_range(0, width / pixels_per_case - 1);
        }
        self.pos_x = rand_x;
        self.pos_y = rand_y;
    }
}

fn collision_count(entity: &(u32, u32), body: &LinkedList<(u32, u32)>) -> usize {
    body.iter().filter(|(x, y)| {
        *x == entity.0 && *y == entity.1
    }).count()
}

fn wait_in_sec(time: u64) {
    let t = time::Duration::from_secs(time);
    thread::sleep(t);
}

fn game_loop(e: &Event, game: &mut Game) -> GameStatus {
    if let Some(r) = e.render_args() {
        game.render(&r);
    }

    if let Some(_u) = e.update_args() {
        if game.update() == false {
            wait_in_sec(3);
            return GameStatus::TitleScreen
        }
    }

    if let Some(k) = e.button_args() {
        if k.state == ButtonState::Press {
            game.pressed(&k.button)
        }
    }
    return GameStatus::Running
}

fn screen_game(e: &Event, game: &mut Game) -> GameStatus {

    if let Some(r) = e.render_args() {
        use graphics;

        game.gl.draw(r.viewport(), |_c, gl| {
            graphics::clear(BACKGROUND_COLOR, gl);
        });
    }

    if let Some(k) = e.button_args() {
        if k.state == ButtonState::Press {
            if k.button == Button::Keyboard(Key::Return) {
                return GameStatus::Running
            }
            else if k.button == Button::Keyboard(Key::Q) {
                return GameStatus::End
            }
        }
    }

    return GameStatus::TitleScreen
}

fn main() {
    let opengl = OpenGL::V3_2;
    let width: u32 = 500;

    let mut window: Window = WindowSettings::new(
        "Best Snake",
        [width, width]
    ).opengl(opengl)
    .exit_on_esc(true)
    .build()
    .unwrap();

    let mut game = Game {
        width: width,
        pixels_per_case: 20,
        gl: GlGraphics::new(opengl),
        score: 0,
        snake: Snake {
            body: LinkedList::from_iter((vec![(3,0), (2,0), (1,0), (0,0)]).into_iter()),
            direction: Direction::Right
        },
        food: Food {
            pos_x: 15,
            pos_y: 15
        },
        status: GameStatus::TitleScreen
    };

    let mut events = Events::new(EventSettings::new()).ups(11);
    while let Some(e) = events.next(&mut window) {

        if game.status == GameStatus::TitleScreen {
            game.status = screen_game(&e, &mut game);
        }

        if game.status == GameStatus::Running {
            game.status = game_loop(&e, &mut game);
        }

        if game.status == GameStatus::End {
            break;
        }

    }
}

// TODO:
// - accelerer jeu au fur et à mesure ?
// - mettre pause
// - afficher le score sur la fenetre
// - ecran de fin
