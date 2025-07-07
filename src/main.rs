use console_engine::{self, Color, ConsoleEngine, KeyCode, pixel};
use euclid::{Point2D, UnknownUnit, Vector2D};

// Engine initialization
const WIDTH: u32 = 17;
const HEIGHT: u32 = 15;
const FPS: u32 = 8;

// Controls
const QUIT_KEY: KeyCode = KeyCode::Char('q');
const PAUSE_KEY: KeyCode = KeyCode::Esc;
const UP_KEY: KeyCode = KeyCode::Up;
const DOWN_KEY: KeyCode = KeyCode::Down;
const LEFT_KEY: KeyCode = KeyCode::Left;
const RIGHT_KEY: KeyCode = KeyCode::Right;

// Colors of the on screen objects
const MAP_COLOR: Color = Color::Green;
const BORDER_COLOR: Color = Color::Black;
const FOOD_COLOR: Color = Color::Red;
const SNEK_COLOR: Color = Color::Blue;
const HEAD_COLOR: Color = Color::Black;

// Characters and strings that will be drawn
const EYE_CHAR: char = '^';
const DEAD_EYE_CHAR: char = 'x';
const GAME_PROMPT: &str = "SNEK";
const PAUSE_PROMPT: &str = "PAUSED";
const SCORE_PROMPT: &str = "SCORE: ";

// Printed at the end of the game
const END_MESSAGE: &str = "Due to your subpar prowess and deriliction of duty, the snek's concept \
of a subjective experience and consciousness has ceased to be...\nFinal Score: ";

// Snek initialization
const STARTING_BODY: [Point; 4] = [
    Point::new(0, 0),
    Point::new(1, 0),
    Point::new(2, 0),
    Point::new(3, 0),
];

// Represents an on scren point and vector
type Point = Point2D<i32, UnknownUnit>;
type Vector = Vector2D<i32, UnknownUnit>;

// Represents the game (the snek and the engine)
struct Game {
    snek: Snek,
    food: Point,
    paused: bool,
    engine: ConsoleEngine,
    width: u32,
    height: u32,
}

impl Game {
    // Creates a new game
    fn new(width: u32, height: u32, fps: u32, starting_body: &[Point]) -> Self {
        Self {
            snek: Snek::new(starting_body),
            food: rand_point(width, height, starting_body),
            paused: false,
            engine: ConsoleEngine::init(width * 2 + 4, height + 2, fps)
                .expect("Console Engine failed to initialize"),
            width,
            height,
        }
    }

    // The main game loop that runs throughout the game
    fn main_loop(&mut self) {
        self.engine.set_title("SNEK");
        while self.snek.alive {
            self.snek.alive = !(self.quit() || self.snek.dead(self.width, self.height));
            self.draw();

            self.engine.draw();
            self.engine.clear_screen();
            self.engine.wait_frame();

            self.input();
            if !self.paused {
                self.snek.slither(&mut self.food, self.width, self.height);
            }
        }
    }

    // Returns the score of the game
    fn score(&self) -> usize {
        self.snek.score()
    }

    // Draws the map, snek, and food
    fn draw(&mut self) {
        self.draw_map();
        self.draw_prompts();
        self.draw_food();
        self.draw_snek();
    }

    // Draws the border and map
    fn draw_map(&mut self) {
        self.engine.fill(pixel::pxl_bg(' ', BORDER_COLOR));
        self.engine.fill_rect(
            2,
            1,
            self.engine.get_width() as i32 - 3,
            self.engine.get_height() as i32 - 2,
            pixel::pxl_bg(' ', MAP_COLOR),
        );
    }

    // Draws the prompts (game, pause, and score)
    fn draw_prompts(&mut self) {
        let score = SCORE_PROMPT.to_owned() + &self.score().to_string();
        let mid = self.engine.get_width() / 2 - score.len() as u32 / 2;
        self.engine.print_fbg(
            mid as i32,
            self.engine.get_height() as i32 - 1,
            &score,
            Color::Reset,
            BORDER_COLOR,
        );
        let prompt = match self.paused {
            true => PAUSE_PROMPT,
            false => GAME_PROMPT,
        };
        let mid = self.engine.get_width() / 2 - prompt.len() as u32 / 2;
        self.engine
            .print_fbg(mid as i32, 0, prompt, Color::Reset, BORDER_COLOR);
    }

    // Draws the food
    fn draw_food(&mut self) {
        self.engine.set_pxl(
            self.food.x * 2 + 2,
            self.food.y + 1,
            pixel::pxl_bg(' ', FOOD_COLOR),
        );
        self.engine.set_pxl(
            self.food.x * 2 + 3,
            self.food.y + 1,
            pixel::pxl_bg(' ', FOOD_COLOR),
        );
    }

    // Draws the snek
    fn draw_snek(&mut self) {
        for part in &self.snek.body {
            self.engine
                .set_pxl(part.x * 2 + 2, part.y + 1, pixel::pxl_bg(' ', SNEK_COLOR));
            self.engine
                .set_pxl(part.x * 2 + 3, part.y + 1, pixel::pxl_bg(' ', SNEK_COLOR));
        }
        let last = self.snek.body.last().unwrap();
        let eye = match self.snek.alive {
            true => EYE_CHAR,
            false => DEAD_EYE_CHAR,
        };
        self.engine.set_pxl(
            last.x * 2 + 2,
            last.y + 1,
            pixel::pxl_fbg(eye, HEAD_COLOR, SNEK_COLOR),
        );
        self.engine.set_pxl(
            last.x * 2 + 3,
            last.y + 1,
            pixel::pxl_fbg(eye, HEAD_COLOR, SNEK_COLOR),
        );
    }

    // Checks if the player wants to quit
    fn quit(&mut self) -> bool {
        self.engine.is_key_pressed(QUIT_KEY)
    }

    // Deals with movement input; returns whether should quit or not
    fn input(&mut self) {
        if self.engine.is_key_pressed(PAUSE_KEY) {
            self.paused = !self.paused;
        } else if self.engine.is_key_pressed(UP_KEY) {
            self.snek.change_direction(Direction::Up);
        } else if self.engine.is_key_pressed(DOWN_KEY) {
            self.snek.change_direction(Direction::Down);
        } else if self.engine.is_key_pressed(LEFT_KEY) {
            self.snek.change_direction(Direction::Left);
        } else if self.engine.is_key_pressed(RIGHT_KEY) {
            self.snek.change_direction(Direction::Right);
        }
    }
}

// Contains information about the snek
struct Snek {
    body: Vec<Point>,
    start_len: usize,
    direction: Direction,
    eating: bool,
    alive: bool,
}

impl Snek {
    // Creates a new snek
    fn new(starting_body: &[Point]) -> Self {
        Self {
            body: Vec::from(starting_body),
            start_len: starting_body.len(),
            direction: Direction::Right,
            eating: false,
            alive: true,
        }
    }

    // Moves the snek in the current direction
    fn slither(&mut self, food: &mut Point, width: u32, height: u32) {
        self.body
            .push(*self.body.last().unwrap() + self.direction.to_vector());
        self.eat(*food);
        if !self.eating {
            self.body.remove(0);
        } else {
            self.eating = false;
            *food = rand_point(width, height, &self.body);
        }
    }

    // Returns whether the snek is dead or not (inside itself or wall)
    fn dead(&mut self, width: u32, height: u32) -> bool {
        let last = self.body.last().unwrap();
        self.body[0..self.body.len() - 1].contains(last)
            || last.x < 0
            || last.y < 0
            || last.x > width as i32 - 1
            || last.y > height as i32 - 1
    }

    // Does some checking and then changes the direction of the snek
    fn change_direction(&mut self, direction: Direction) {
        if self.direction != direction.opposite() {
            self.direction = direction;
        }
    }

    // Returns the score (current len - starting len)
    fn score(&self) -> usize {
        self.body.len() - self.start_len
    }

    // Elongates the snek if its head is on a food point
    fn eat(&mut self, food: Point) {
        if *self.body.last().unwrap() == food {
            self.eating = true;
        }
    }
}

// Represents one of the four directions
#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    // Returns the opposite direction
    fn opposite(&self) -> Self {
        match self {
            Direction::Up => Self::Down,
            Direction::Down => Self::Up,
            Direction::Left => Self::Right,
            Direction::Right => Self::Left,
        }
    }

    // Converts the direction to a vector
    fn to_vector(&self) -> Vector {
        match self {
            Direction::Up => Vector::new(0, -1),
            Direction::Down => Vector::new(0, 1),
            Direction::Left => Vector::new(-1, 0),
            Direction::Right => Vector::new(1, 0),
        }
    }
}

// Randomizes a point, excluding a list points
fn rand_point(width: u32, height: u32, exclude: &[Point]) -> Point {
    let mut point = Point::new(
        fastrand::i32(0..width as i32),
        fastrand::i32(0..height as i32),
    );
    while exclude.contains(&point) {
        point = Point::new(
            fastrand::i32(0..width as i32),
            fastrand::i32(0..height as i32),
        );
    }
    point
}

// Entry point
fn main() {
    let mut game = Game::new(WIDTH, HEIGHT, FPS, &STARTING_BODY);
    game.main_loop();
    let score = game.score();
    drop(game);
    println!("{}", END_MESSAGE.to_string() + &score.to_string());
}
