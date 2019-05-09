extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL, GlyphCache };
use graphics::{Transformed, math};

use std::collections::LinkedList;
use std::iter::FromIterator;
use rand::Rng;
use std::{thread, time};

static BACKGROUND_COLOR: [f32; 4] = [0.56, 0.93, 0.56, 1.0];
static SNAKE_HEAD_COLOR: [f32; 4] = [0.6, 0.2, 0.6, 1.0];
static SNAKE_BODY_COLOR: [f32; 4] = [0.925, 0.0, 0.55, 1.0];
static FOOD_COLOR: [f32; 4] = [1.0, 1.0, 0.88, 1.0];
static MAIN_TEXT_COLOR: [f32; 4] = [1.0, 1.0, 0.88, 1.0];
static TITLE_TEXT_COLOR: [f32; 4] = SNAKE_BODY_COLOR;
static END_SCREEN_COLOR: [f32; 4] = [1.0, 1.0, 0.88, 0.2];

#[derive(Debug, Clone, PartialEq)]
enum GameStatus {
    Running,
    TitleScreen,
    EndScreen,
    Quit,
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
    food: Food
}

impl Game {
    fn render(&mut self, arg: &RenderArgs) {

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
            if let Some(&first) = squares.first() {
                squares.remove(0);
                squares.into_iter().for_each(|square| graphics::rectangle(SNAKE_BODY_COLOR, square, c.transform, gl));
                graphics::rectangle(SNAKE_HEAD_COLOR, first, c.transform, gl);
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

        let square = graphics::rectangle::square(
            (self.pos_x * pixels) as f64,
            (self.pos_y * pixels) as f64,
            pixels as f64);

        gl.draw(args.viewport(), |c, gl| {
            graphics::rectangle(FOOD_COLOR, square, c.transform, gl);
        });
    }

    pub fn update(&mut self, width: u32, pixels_per_case: u32, snake: &Snake) {
        let mut rand_x = rand::thread_rng().gen_range(0, width / pixels_per_case);
        let mut rand_y = rand::thread_rng().gen_range(0, width / pixels_per_case);
        while collision_count(&(rand_x, rand_y), &snake.body) != 0 {
            rand_x = rand::thread_rng().gen_range(0, width / pixels_per_case);
            rand_y = rand::thread_rng().gen_range(0, width / pixels_per_case);
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

fn draw_text(glyph_cache: &mut GlyphCache, color: [f32; 4], size: u32, text: &str, transform: math::Matrix2d, gl: &mut GlGraphics) {
    graphics::text(
        color,
        size,
        text,
        glyph_cache,
        transform,
        gl,
    ).unwrap();
}

fn wait_in_sec(time: u64) {
    let t = time::Duration::from_secs(time);
    thread::sleep(t);
}

fn game_events_manager(e: &Event, game: &mut Game) -> GameStatus {
    if let Some(r) = e.render_args() {
        game.render(&r);
    }

    if let Some(_u) = e.update_args() {
        if game.update() == false {
            return GameStatus::EndScreen
        }
    }

    if let Some(k) = e.button_args() {
        if k.state == ButtonState::Press {
            game.pressed(&k.button)
        }
    }
    return GameStatus::Running
}

fn title_screen_events_manager(e: &Event, game: &mut Game, glyph_cache: &mut GlyphCache) -> GameStatus {

    if let Some(r) = e.render_args() {
        let pixels = game.pixels_per_case;
        game.gl.draw(r.viewport(), |c, gl| {

            graphics::clear(BACKGROUND_COLOR, gl);

            draw_text(
                glyph_cache,
                TITLE_TEXT_COLOR,
                50,
                "SNAKE",
                c.transform.trans(8.3 * pixels as f64, 5.5 * pixels as f64),
                gl
            );

            draw_text(
                glyph_cache,
                MAIN_TEXT_COLOR,
                16,
                "Press Enter to play",
                c.transform.trans(7.3 * pixels as f64, 13.0 * pixels as f64),
                gl
            );

            draw_text(
                glyph_cache,
                MAIN_TEXT_COLOR,
                13,
                "s: scores table (wip)",
                c.transform.trans(1.0 * pixels as f64, 23.0 * pixels as f64),
                gl
            );

            draw_text(
                glyph_cache,
                MAIN_TEXT_COLOR,
                13,
                "q: quit",
                c.transform.trans(1.0 * pixels as f64, 24.0 * pixels as f64),
                gl
            );

        });
    }

    if let Some(k) = e.button_args() {
        if k.state == ButtonState::Press {

            match k.button {
                Button::Keyboard(Key::Return) => return GameStatus::Running,
                Button::Keyboard(Key::Q) => return GameStatus::Quit,
                _ => return GameStatus::TitleScreen
            }
        }
    }

    return GameStatus::TitleScreen
}

fn end_screen_events_manager(e: &Event, game: &mut Game, wait_end_screen: &mut u32, glyph_cache: &mut GlyphCache) -> GameStatus {
    if let Some(r) = e.render_args() {
        *wait_end_screen += 1;

        let pixels = game.pixels_per_case;
        game.gl.draw(r.viewport(), |c, gl| {
            let background = graphics::rectangle::square(0.0, 0.0, 500.0);
            graphics::rectangle(END_SCREEN_COLOR, background, c.transform, gl);
            draw_text(
                glyph_cache,
                TITLE_TEXT_COLOR, 35,
                "Game over",
                c.transform.trans(7.0 * pixels as f64, 12.0 * pixels as f64),
                gl
            );
        });

        if *wait_end_screen == 2 {
            wait_in_sec(2);
            *wait_end_screen = 0;
            return GameStatus::TitleScreen
        }
    }
    return GameStatus::EndScreen
}

fn new_game(width: u32, opengl: OpenGL) -> Game {
    Game {
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
        }
    }
}

fn main() {
    let opengl = OpenGL::V3_2;
    let width: u32 = 500;

    let mut window: Window = WindowSettings::new(
        "Best Snake Ever",
        [width, width]
    ).opengl(opengl)
    .exit_on_esc(true)
    .build()
    .unwrap();

    let mut glyph_cache: GlyphCache = GlyphCache::new(
        "./assets/PxPlus_IBM_VGA8.ttf",
        (),
        opengl_graphics::TextureSettings::new()
    ).expect("Unable to load font");

    let mut status = GameStatus::TitleScreen;
    let mut wait_end_screen: u32 = 0;
    let mut game = new_game(width, opengl);

    let mut events = Events::new(EventSettings::new()).ups(11);
    while let Some(e) = events.next(&mut window) {

        if status == GameStatus::TitleScreen {
            status = title_screen_events_manager(&e, &mut game, &mut glyph_cache);
            if status == GameStatus::Running {
                game = new_game(width, opengl);
            }
        }

        if status == GameStatus::Running {
            status = game_events_manager(&e, &mut game);
        }

        if status == GameStatus::EndScreen {
            status = end_screen_events_manager(&e, &mut game, &mut wait_end_screen, &mut glyph_cache);
        }

        if status == GameStatus::Quit {
            break;
        }

    }
}

// TODO:
// - mettre pause
// - afficher le score sur la fenetre
// - ecran de fin
// - enlever les liste chainées, mettre des vecteurs
// - pb des keys qui vont plus vite que l'update
// - verifier random de la food
// - bloquer resize
