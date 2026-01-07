// This line imports modules from the `crossterm` crate, which helps us manipulate the terminal.
// Crates are like libraries or packages in other languages.
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand, QueueableCommand,
};
// We need the `Rng` trait to generate random numbers for the food position.
use rand::Rng;
// Standard library imports for collections, input/output, and time management.
use std::{
    collections::{HashSet, VecDeque},
    io::{self, Write},
    time::{Duration, Instant},
};

// Constants determine the size of our game board.
// `u16` means an unsigned 16-bit integer (can't be negative).
const WIDTH: u16 = 40;
const HEIGHT: u16 = 20;

// Structs define custom data types to group related data.
// #[derive(...)] asks the compiler to automatically implement basic behaviors for us.
// - Clone/Copy: Allows us to duplicate this Point easily.
// - PartialEq/Eq: Allows us to compare two Points with `==`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Point {
    x: u16,
    y: u16,
}

// Enums allow us to define a type that can be one of several variants.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// We can add methods to our types using `impl`.
impl Direction {
    // A simple helper to get the opposite direction, used to prevent 180-degree turns.
    // `&self` means this method borrows the Direction instance safely.
    // `-> Self` means it returns a new Direction.
    /* 
       Note: We removed the 'opposite' function in a previous step as it was unused, 
       but for a tutorial, it's good to know we *could* put logic here!
       We will handle direction logic directly in the input loop for simplicity.
    */
}

// The core Game state struct.
struct Game {
    // VecDeque is a "double-ended queue". It's like a list but efficient at adding/removing
    // items from both the front and the back. Perfect for a snake!
    snake: VecDeque<Point>,
    food: Point,
    obstacles: HashSet<Point>,
    direction: Direction,
    score: usize,
    level: u32,
    game_over: bool,
    width: u16,
    height: u16,
}

impl Game {
    // Constructor method to create a new Game instance.
    fn new(width: u16, height: u16) -> Self {
        // Start the snake in the middle of the screen.
        let start_x = width / 2;
        let start_y = height / 2;

        // Create the initial snake body parts.
        // `mut` means this variable is mutable (can be changed). By default, variables are immutable in Rust!
        let mut snake = VecDeque::new();
        snake.push_back(Point { x: start_x, y: start_y });
        snake.push_back(Point {
            x: start_x - 1,
            y: start_y,
        });
        snake.push_back(Point {
            x: start_x - 2,
            y: start_y,
        });

        let mut game = Game {
            snake,
            food: Point { x: 0, y: 0 },
            obstacles: HashSet::new(),
            direction: Direction::Right,
            score: 0,
            level: 1,
            game_over: false,
            width,
            height,
        };
        
        game.spawn_food();
        game
    }

    // Function to place food in a random location not occupied by the snake.
    // `&mut self` means this method needs to modify the Game state.
    fn spawn_food(&mut self) {
        let mut rng = rand::thread_rng(); // Get a random number generator thread.
        loop {
            // Generate random x and y coordinates within the walls.
            let x = rng.gen_range(1..self.width - 1);
            let y = rng.gen_range(1..self.height - 1);
            let point = Point { x, y };
            
            // If the generated point is NOT inside the snake body or obstacles, we found a valid spot!
            if !self.snake.contains(&point) && !self.obstacles.contains(&point) {
                self.food = point;
                break; // Exit the loop.
            }
        }
    }

    // Generate random obstacles for the current level
    fn generate_level(&mut self) {
        let mut rng = rand::thread_rng();
        self.obstacles.clear();
        
        // Number of obstacles increases with level
        let num_obstacles = self.level * 3 + 5;
        
        for _ in 0..num_obstacles {
            // Randomly choose vertical or horizontal wall
            let is_horizontal = rng.gen_bool(0.5);
            let length = rng.gen_range(3..8);
            
            let start_x = rng.gen_range(2..self.width - 2);
            let start_y = rng.gen_range(2..self.height - 2);
            
            for i in 0..length {
                let p = if is_horizontal {
                    Point { x: start_x + i, y: start_y }
                } else {
                    Point { x: start_x, y: start_y + i }
                };
                
                // Keep obstacles within bounds and away from snake/food
                if p.x > 0 && p.x < self.width - 1 
                   && p.y > 0 && p.y < self.height - 1
                   && !self.snake.contains(&p)
                   && p != self.food 
                   // Ensure we don't spawn right in front of the snake's current path (simple check)
                   && self.snake.front().map_or(true, |head| (head.x as i32 - p.x as i32).abs() + (head.y as i32 - p.y as i32).abs() > 3)
                {
                    self.obstacles.insert(p);
                }
            }
        }
    }

    // Update the game state (move snake, check collisions).
    fn update(&mut self) {
        if self.game_over {
            return;
        }

        // Calculate the new head position based on current direction.
        // `unwrap()` is used because we know the snake is never empty. If it was empty, this would crash!
        let head = self.snake.front().unwrap();
        
        // `match` is like a powerful switch statement. It forces us to handle every possibility.
        let new_head = match self.direction {
            Direction::Up => Point {
                x: head.x,
                // wrapping_sub handles subtraction that might go below 0 (though we check walls later).
                y: head.y.wrapping_sub(1), 
            },
            Direction::Down => Point {
                x: head.x,
                y: head.y + 1,
            },
            Direction::Left => Point {
                x: head.x.wrapping_sub(1),
                y: head.y,
            },
            Direction::Right => Point {
                x: head.x + 1,
                y: head.y,
            },
        };

        // Wall collision checks.
        // If the head hits the borders (0 or width/height limit), game over.
        if new_head.x == 0
            || new_head.x >= self.width - 1
            || new_head.y == 0
            || new_head.y >= self.height - 1
        {
            self.game_over = true;
            return;
        }

        // Self collision check.
        // If the snake already contains the new head position, we bit ourselves!
        if self.snake.contains(&new_head) {
             self.game_over = true;
            return;   
        }

        // Obstacle collision check
        if self.obstacles.contains(&new_head) {
            self.game_over = true;
            return;
        }

        // Move the snake:
        // 1. Add the new head position to the front of the deque.
        self.snake.push_front(new_head);

        // 2. Check if we ate food.
        if new_head == self.food {
            // Ate food: Score goes up, spawn new food.
            self.score += 1;
            self.spawn_food();
            
            // Level Up Check
            if self.score % 5 == 0 {
                self.level += 1;
                self.generate_level();
            }
            // IMPORTANT: We do NOT remove the tail. This makes the snake grow by 1 block!
        } else {
            // Didn't eat: Remove the last block (tail) to maintain the same length.
            // This creates the illusion of movement.
            self.snake.pop_back();
        }
    }

    // Render the current state to the terminal.
    // `stdout` is a "handle" to the terminal output.
    // Returns `io::Result<()>` because writing to console could theoretically fail.
    fn draw(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        // Draw Borders
        // We queue up commands instead of running them one by one for performance.
        stdout.queue(SetForegroundColor(Color::Grey))?; // Set color (notice the `?` to handle potential errors)
        
        for x in 0..self.width {
            // Draw top and bottom walls
            stdout
                .queue(MoveTo(x, 0))?
                .queue(Print("█"))?
                .queue(MoveTo(x, self.height - 1))?
                .queue(Print("█"))?;
        }
        for y in 0..self.height {
            // Draw left and right walls
            stdout
                .queue(MoveTo(0, y))?
                .queue(Print("█"))?
                .queue(MoveTo(self.width - 1, y))?
                .queue(Print("█"))?;
        }

        // Draw Obstacles
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        for obstacle in &self.obstacles {
            stdout
                .queue(MoveTo(obstacle.x, obstacle.y))?
                .queue(Print("▓"))?;
        }

        // Draw Score
        stdout
            .queue(MoveTo(2, 0))?
            .queue(MoveTo(2, 0))?
            .queue(Print(format!(" Score: {}  Level: {} ", self.score, self.level)))?;

        // Draw Food
        stdout
            .queue(SetForegroundColor(Color::Red))?
            .queue(MoveTo(self.food.x, self.food.y))?
            .queue(Print("●"))?;

        // Draw Snake
        stdout.queue(SetForegroundColor(Color::Green))?;
        for (i, point) in self.snake.iter().enumerate() {
            stdout.queue(MoveTo(point.x, point.y))?;
            if i == 0 {
                stdout.queue(Print("O"))?; // Head
            } else {
                stdout.queue(Print("o"))?; // Body
            }
        }
        
        // Reset color to default so we don't mess up the terminal
        stdout.queue(ResetColor)?;
        Ok(())
    }
}

// Struct to handle cleanup when the program exits.
// Rust has a feature called "RAII" (Resource Acquisition Is Initialization).
// When variables go "out of scope" (are no longer needed), their `drop` function is called.
struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        // These commands run automatically when `_cleanup` variable dies at the end of main().
        // We make sure to restore the terminal to normal mode.
        let _ = disable_raw_mode();
        let _ = io::stdout().execute(Show); // Show the cursor again
        let _ = io::stdout().execute(LeaveAlternateScreen); // Switch back to normal terminal buffer
    }
}

// The main entry point of our program.
fn main() -> io::Result<()> {
    // Create our cleanup guard. It does nothing now, but when `main` finishes, its `drop` runs!
    let _cleanup = CleanUp;
    
    // Enable "raw mode". This lets us read keys directly (like 'w' instead of 'w'+Enter).
    enable_raw_mode()?;
    
    let mut stdout = io::stdout();
    // Use an "alternate screen" buffer so we don't clutter the user's terminal history.
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?; // Hide the flashing cursor cursor

    // Initialize the game state
    let mut game = Game::new(WIDTH, HEIGHT);
    
    // Timer for our game loop
    let mut last_frame = Instant::now();
    let tick_rate = Duration::from_millis(150); // Game updates every 150ms

    // Infinite game loop
    loop {
        // --- Input Handling ---
        // `poll` checks if there is an input event waiting, without blocking forever.
        // We wait for 0ms (check instantly).
        if event::poll(Duration::from_millis(0))? {
            // Read the event
            if let Event::Key(key) = event::read()? {
                match key.code {
                    // Quit on 'q', 'Esc', or Ctrl+C
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    
                    // Change direction based on key press
                    // We check `game.direction` to prevent moving directly backwards (e.g. going Left while moving Right)
                    KeyCode::Left | KeyCode::Char('a') => {
                        if game.direction != Direction::Right {
                            game.direction = Direction::Left;
                        }
                    }
                    KeyCode::Right | KeyCode::Char('d') => {
                        if game.direction != Direction::Left {
                            game.direction = Direction::Right;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('w') => {
                        if game.direction != Direction::Down {
                            game.direction = Direction::Up;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('s') => {
                        if game.direction != Direction::Up {
                            game.direction = Direction::Down;
                        }
                    }
                    _ => {} // Ignore other keys
                }
            }
        }

        // --- Game Update & Rendering ---
        // Check if enough time has passed to update the game frame
        if last_frame.elapsed() >= tick_rate {
            game.update();
            last_frame = Instant::now();
            
            // Clear the screen buffer before drawing the new frame.
            stdout.queue(Clear(ClearType::All))?; 
            
            if !game.game_over {
                 game.draw(&mut stdout)?;
            } else {
                 // Draw Game Over Screen
                 let msg = "GAME OVER";
                 let score_msg = format!("Final Score: {}", game.score);
                 let restart_msg = "Press Q to Quit";
                 
                 let center_x = WIDTH / 2;
                 let center_y = HEIGHT / 2;
                 
                 stdout.queue(SetForegroundColor(Color::Red))?;
                 stdout.queue(MoveTo(center_x - (msg.len() as u16 / 2), center_y - 1))?;
                 stdout.queue(Print(msg))?;
                 
                 stdout.queue(SetForegroundColor(Color::White))?;
                 stdout.queue(MoveTo(center_x - (score_msg.len() as u16 / 2), center_y + 1))?;
                 stdout.queue(Print(score_msg))?;
                 
                 stdout.queue(MoveTo(center_x - (restart_msg.len() as u16 / 2), center_y + 3))?;
                 stdout.queue(Print(restart_msg))?;
                 stdout.queue(ResetColor)?;
            }
            
            // Flush commands to the terminal (actually draw everything now).
            // This buffering prevents flickering.
            stdout.flush()?;
        }
        
        // Loop Logic for Game Over
        if game.game_over {
             // If game over, we just wait for the user to quit.
             // We poll a bit slower to save CPU.
             if event::poll(Duration::from_millis(100))? {
                 if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        _ => {}
                    }
                 }
             }
        } else {
            // Small sleep to prevent 100% CPU usage looping while waiting for the next frame.
             let elapsed = last_frame.elapsed();
             if elapsed < tick_rate {
                 std::thread::sleep(Duration::from_millis(10));
             }
        }
    }

    Ok(()) // Return "Ok" to signal the program finished successfully.
}