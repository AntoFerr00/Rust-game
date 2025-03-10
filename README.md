# Little Super Mario Rust Game

A fun 2D arcade-style game built using the [Bevy](https://bevyengine.org/) game engine in Rust. In this game, you control a player character that can move, jump, and defeat enemies while avoiding obstacles. Enemies roam the level with random movements and wrap around the screen. Defeat all enemies to win the game, or be careful—if you get hit by an enemy, it's game over!

## Features

- **Player Movement:**  
  Move left or right and jump to navigate the level.

- **Enemy Behavior:**  
  Enemies move horizontally with random speeds and directions. They reverse direction upon hitting obstacles, making them challenging targets.

- **Obstacles:**  
  Randomly spawned obstacles add additional difficulty by blocking paths.

- **Collision Detection:**  
  Simple collision logic determines when the player stomps an enemy, hits an obstacle, or suffers a game over from a side hit.

- **Win & Lose Conditions:**  
  - **Win:** When all enemies are defeated, a win screen is displayed.
  - **Lose:** If the player is hit by an enemy (except when stomping from above), the game ends with a game over screen.

- **UI Score Display:**  
  The game keeps track of your score as you defeat enemies.

## Requirements

- **Rust:** Latest stable version recommended. Install from [rustup.rs](https://rustup.rs/).
- **Bevy Engine:** For game development in Rust.  
- **rand Crate:** For random number generation.

## Installation

1. **Clone the Repository:**

   ```bash
   git clone https://github.com/yourusername/my_bevy_game.git
   cd my_bevy_game
   ```
2. **Install Rust (if not already installed):**

Install Rust using rustup by following the instructions on rustup.rs.

3. **Install Dependencies:**

The required dependencies are specified in your Cargo.toml. In your project directory, run:

```bash
cargo build
```
This will download and compile all necessary dependencies, including Bevy and rand.

## Running the Game

To start the game, simply run:

```bash
cargo run
```
A game window should open, and you can control the player using the keyboard.

## Game Controls

- Left / A: Move left
- Right / D: Move right
- Space / Key2: Jump

## Project Structure

- main.rs:
Contains the game loop, system definitions, and overall logic using Bevy's ECS (Entity Component System).

- Components & Resources:

Player, Enemy, Obstacle, and Ground components define game entities.
Velocity is used for moving entities.
Gravity, Score, and GroundData are resources that control game physics and state.

- Systems:
Various systems manage input, physics (gravity & movement), collision detection, enemy behavior, UI updates, and game state (win/lose conditions).

## Contributing

Contributions, suggestions, and bug reports are welcome!
Feel free to open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License. See the LICENSE file for details.

