mod colorize;
#[cfg(target_os = "horizon")]
mod touch_input;

use colorize::Stylize;
#[cfg(target_os = "horizon")]
use touch_input::TouchInputHandler;
#[cfg(not(target_os = "horizon"))]
use clap::Parser;
#[cfg(not(target_os = "horizon"))]
use crossterm::{
    cursor,
    event,
    event::{Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal,
    terminal::{ClearType}
};
use regex::regex;
use std::{
    fs,
    io::{stdout, Write},
    path::{PathBuf}
};
#[cfg(target_os = "horizon")]
use ctru::prelude::*;

#[cfg(not(target_os = "horizon"))]
#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'f', long = "file", help = "Path to a Sokoban puzzle set")]
    file: Option<String>,
}

#[derive(Clone)]
struct Map {
    name: String,
    width: i32,
    height: i32,
    player: Player,
    cubes: Vec<Cube>,
    buttons: Vec<Button>,
    finish: Finish,
    walls: Vec<Wall>,
}

#[derive(Clone)]
struct Player {
    x: i32,
    y: i32,
    location_history: Vec<(i32, i32)>,
}

#[derive(Clone)]
struct Cube {
    x: i32,
    y: i32,
    location_history: Vec<(i32, i32)>,
}

#[derive(Clone)]
struct Button {
    x: i32,
    y: i32,
}

#[derive(Clone)]
struct Finish {
    x: i32,
    y: i32,
}

#[derive(Clone)]
struct Wall {
    x: i32,
    y: i32,
}

impl Map {
    fn try_move_player(&mut self, direction: char) {
        let mut new_x = self.player.x;
        let mut new_y = self.player.y;

        let player_old = (self.player.x, self.player.y);
        let cubes_old: Vec<(i32, i32)> = self.cubes.iter().map(|c| (c.x, c.y)).collect();

        match direction {
            'u' => new_y = self.player.y - 1,
            'd' => new_y = self.player.y + 1,
            'l' => new_x = self.player.x - 1,
            'r' => new_x = self.player.x + 1,
            _ => {}
        }

        if new_x < 0 || new_x >= self.width ||
            new_y < 0 || new_y >= self.height {
            return
        }

        if self.walls.iter().any(|wall| wall.x == new_x && wall.y == new_y) {
            return
        }

        let cube_index = self.cubes.iter().position(|cube| {
            cube.x == new_x && cube.y == new_y
        });

        if let Some(index) = cube_index {
            if !self.try_move_cube(index, direction) {
                return
            }
        }

        self.player.location_history.push(player_old);

        for (cube, old) in self.cubes.iter_mut().zip(cubes_old) {
            cube.location_history.push(old);
        }

        self.player.x = new_x;
        self.player.y = new_y;
    }

    fn try_move_cube(&mut self, index: usize, direction: char) -> bool {
        let cube = &self.cubes[index];

        let mut new_x = cube.x;
        let mut new_y = cube.y;

        match direction {
            'u' => new_y -= 1,
            'l' => new_x -= 1,
            'd' => new_y += 1,
            'r' => new_x += 1,
            _ => {}
        }

        if new_x < 0 || new_x >= self.width ||
            new_y < 0 || new_y >= self.height {
            return false
        }

        for (i, other_cube) in self.cubes.iter().enumerate() {
            if i != index &&
                other_cube.x == new_x &&
                other_cube.y == new_y {
                return false
            }
        }

        if self.walls.iter().any(|wall| wall.x == new_x && wall.y == new_y) {
            return false
        }

        self.cubes[index].x = new_x;
        self.cubes[index].y = new_y;

        true
    }

    fn undo(&mut self) {
        if let Some((x, y)) = self.player.location_history.pop() {
            self.player.x = x;
            self.player.y = y;
        }

        for cube in self.cubes.iter_mut() {
            if let Some((x, y)) = cube.location_history.pop() {
                cube.x = x;
                cube.y = y;
            }
        }
    }

    fn all_objectives_met(&self) -> bool {
        self.finish_reached() && self.all_buttons_pressed()
    }

    fn finish_reached(&self) -> bool {
        if (self.player.x != self.finish.x || self.player.y != self.finish.y) && self.has_finish() {
            false
        } else {
            true
        }
    }

    fn all_buttons_pressed(&self) -> bool {
        self.buttons.iter().all(|button| {
            self.cubes.iter().any(|cube| {
                cube.x == button.x && cube.y == button.y
            })
        })
    }

    fn has_buttons(&self) -> bool {
        if self.buttons.len() > 0 {
            true
        } else {
            false
        }
    }

    fn has_cubes(&self) -> bool {
        if self.cubes.len() > 0 {
            true
        } else {
            false
        }
    }

    fn has_finish(&self) -> bool {
        if self.finish.x != 0 && self.finish.y != 0 {
            true
        } else {
            false
        }
    }
}

fn load_maps(_filepath: &PathBuf) -> Vec<Map> {
    let contents = if *_filepath == PathBuf::default() {
        include_str!("microban.txt").to_string()
    } else {
        fs::read_to_string(_filepath).expect("Failed to read file")
    };

    contents
        .split(';')
        .filter_map(|level| {
            let mut lines = level.lines();

            let header = lines.find(|line| !line.trim().is_empty())?;

            let map_lines: Vec<&str> = lines
                .filter(|line| !line.trim().is_empty())
                .collect();

            if map_lines.is_empty() {
                return None
            }

            Some(parse_map(header, &map_lines))
        })
        .collect()
}

fn parse_map(header: &str, unparsed_lines: &[&str]) -> Map {
    let name;
    let mut lines: Vec<&str> = unparsed_lines.to_vec(); // default: just lines

    if regex!(r"[^#\s]+").is_match(header) {
        name = header.trim().to_string();
    } else {
        name = "".to_string();
        lines = std::iter::once(header)
            .chain(lines.iter().copied())
            .collect();
    }

    let height = lines.len() as i32;
    let width = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0) as i32;

    let mut player = Player {
        x: 0,
        y: 0,
        location_history: Vec::new(),
    };

    let mut finish = Finish {
        x: 0,
        y: 0,
    };

    let mut cubes = Vec::new();
    let mut buttons = Vec::new();
    let mut walls = Vec::new();

    for (y, line) in lines.iter().enumerate() {
        for (x, tile) in line.chars().enumerate() {
            let x = x as i32;
            let y = y as i32;

            match tile {
                '#' => {
                    walls.push(Wall { x, y });
                }
                '@' => {
                    player.x = x;
                    player.y = y;
                }
                '$' => {
                    cubes.push(Cube {
                        x,
                        y,
                        location_history: Vec::new(),
                    });
                }
                '.' => {
                    buttons.push(Button { x, y });
                }
                '€' => {
                    finish.x = x;
                    finish.y = y;
                }
                '*' => {
                    cubes.push(Cube {
                        x,
                        y,
                        location_history: Vec::new(),
                    });

                    buttons.push(Button { x, y });
                }
                '+' => {
                    player.x = x;
                    player.y = y;

                    buttons.push(Button { x, y });
                }
                ' ' => {}
                _ => {}
            }
        }
    }

    Map {
        name,
        width,
        height,
        player,
        cubes,
        buttons,
        finish,
        walls,
    }
}

fn render(map: &Map) -> String {
    let mut out = String::new();

    for y in 0..map.height {
        for x in 0..map.width {
            let has_player = {
                x == map.player.x && y == map.player.y
            };

            let has_cube = map.cubes.iter().any(|cube| {
                cube.x == x && cube.y == y
            });

            let has_button = map.buttons.iter().any(|button| {
                button.x == x && button.y == y
            });

            let has_finish = {
                x == map.finish.x && y == map.finish.y && x != 0 && y != 0
            };

            let has_wall = map.walls.iter().any(|wall| {
                wall.x == x && wall.y == y
            });

            let ch = if has_player && has_finish {
                'K'.green().to_string()
            } else if has_player {
                'K'.yellow().to_string()
            } else if has_cube && has_button {
                'O'.green().to_string()
            } else if has_cube {
                'o'.blue().to_string()
            } else if has_button {
                'x'.red().to_string()
            } else if has_finish {
                if map.all_buttons_pressed() {
                    'F'.blue().to_string()
                } else {
                    'F'.red().to_string()
                }
            } else if has_wall {
                '#'.grey().to_string()
            } else {
                ' '.to_string()
            };
            out.push_str(&ch)
        }
        out.push('\r');
        out.push('\n');
    }
    out
}

fn format_help_text(map: &Map) -> String {
    match (map.has_finish(), map.has_cubes(), map.has_buttons()) {
        (true, true, true) => {
            format!(
                "{}: Player    {}: Cube\r\n{}: Button    {}: Cube on Button\r\n{}: Wall      {}: Finish",
                'K'.yellow(), 'o'.blue(), 'x'.red(), 'O'.green(), '#'.grey(), 'F'.blue()
            )
        }
        (false, true, true) => {
            format!(
                "{}: Player    {}: Cube\r\n{}: Button    {}: Cube on Button\r\n{}: Wall\r",
                'K'.yellow(), 'o'.blue(), 'x'.red(), 'O'.green(), '#'.grey()
            )
        }
        (true, false, true) => {
            format!(
                "{}: Player    {}: Button\r\n{}: Wall      {}: Finish",
                'K'.yellow(), 'x'.red(), '#'.grey(), 'F'.blue()
            )
        }
        (true, true, false) => {
            format!(
                "{}: Player    {}: Cube\r\n{}: Wall      {}: Finish",
                'K'.yellow(), 'o'.blue(), '#'.grey(), 'F'.blue()
            )
        }
        (false, false, true) => {
            format!(
                "{}: Player    {}: Button\r\n{}: Wall",
                'K'.yellow(), 'x'.red(), '#'.grey()
            )
        }
        (false, true, false) => {
            format!(
                "{}: Player    {}: Cube\r\n{}: Wall",
                'K'.yellow(), 'o'.blue(), '#'.grey()
            )
        }
        (true, false, false) => {
            format!(
                "{}: Player    {}: Wall\r\n{}: Finish",
                'K'.yellow(), '#'.grey(), 'F'.blue()
            )
        }
        (false, false, false) => {
            format!(
                "{}: Player    {}: Wall\r",
                'K'.yellow(), '#'.grey()
            )
        }
    }
}

#[cfg(target_os = "horizon")]
fn main() {
    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let gfx = Gfx::new().unwrap();
    let top_screen = Console::new(gfx.top_screen.borrow_mut());
    let bottom_screen = Console::new(gfx.bottom_screen.borrow_mut());

    let maps = load_maps(&Default::default());
    let mut map_index = 0;
    let mut map = maps[map_index].clone();
    let mut dirty = true;
    let mut touch_handler = TouchInputHandler::new();

    while apt.main_loop() {
        if dirty {
            top_screen.select();
            print!("\x1b[0;0HLevel: {}\r\n{}\r\n", map.name, render(&map));
            stdout().flush().unwrap();

            bottom_screen.select();
            let help_text = format_help_text(&map);

            if !map.all_objectives_met() {
                print!(
                    "\x1b[0;0H\r\nD-pad, Circle Pad or Swipe to move\r\nPress Y to undo\r\nPress X to reset\r\nPress Start to quit\r\nMoves: {}      \r\n\x1b[27;0H{}\r\n",
                    map.player.location_history.len(), help_text
                );
            } else {
                let next_msg = if map_index >= maps.len() - 1 {
                    "Press A to go back to the first level"
                } else {
                    "Press A to go to the next level"
                };
                print!(
                    "\x1b[2J\x1b[0;0H\r\nYou did it!\r\n{}\r\nPress X to reset\r\nPress Start to quit\r\nMoves: {}      \r\n\x1b[27;0H{}\r\n",
                    next_msg, map.player.location_history.len(), help_text
                );
            }
            stdout().flush().unwrap();
            dirty = false;
        }

        gfx.wait_for_vblank();
        hid.scan_input();

        if let Some(direction) = touch_handler.update(&hid) {
            if !map.all_objectives_met() {
                map.try_move_player(direction.as_char());
                dirty = true;
            }
        }

        let keys = hid.keys_down();

        if (keys.contains(KeyPad::DPAD_UP) || keys.contains(KeyPad::CPAD_UP)) && !map.all_objectives_met() {
            map.try_move_player('u');
            dirty = true;
        } else if (keys.contains(KeyPad::DPAD_DOWN) || keys.contains(KeyPad::CPAD_DOWN)) && !map.all_objectives_met() {
            map.try_move_player('d');
            dirty = true;
        } else if (keys.contains(KeyPad::DPAD_LEFT) || keys.contains(KeyPad::CPAD_LEFT)) && !map.all_objectives_met() {
            map.try_move_player('l');
            dirty = true;
        } else if (keys.contains(KeyPad::DPAD_RIGHT) || keys.contains(KeyPad::CPAD_RIGHT)) && !map.all_objectives_met() {
            map.try_move_player('r');
            dirty = true;
        } else if keys.contains(KeyPad::Y) && !map.all_objectives_met() {
            map.undo();
            dirty = true;
        } else if keys.contains(KeyPad::X) {
            map = maps[map_index].clone();
            top_screen.select();
            print!("\x1b[2J");
            bottom_screen.select();
            print!("\x1b[2J");
            dirty = true;
        } else if keys.contains(KeyPad::A) && map.all_objectives_met() {
            map_index = (map_index + 1) % maps.len();
            map = maps[map_index].clone();
            top_screen.select();
            print!("\x1b[2J");
            bottom_screen.select();
            print!("\x1b[2J");
            dirty = true;
        } else if keys.contains(KeyPad::START) {
            break;
        }
    }
}

#[cfg(not(target_os = "horizon"))]
fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let maps = match args.file {
        Some(file) => load_maps(&PathBuf::from(file)),
        None => load_maps(&Default::default())
    };

    let mut map_index = 0;

    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    let mut map = maps[map_index].clone();

    let mut dirty = true;

    let result = (|| -> std::io::Result<()> {
        loop {
            if dirty {
                execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                let help_text = format_help_text(&map);
                if !(map.has_cubes() || map.has_finish()){
                    map_index += 1;
                    if map_index >= maps.len() {
                        map_index = 0;
                    }
                    map = maps[map_index].clone();
                    dirty = true;
                }
                if !map.all_objectives_met() {
                    print!("Level: {}\r\n{}\r\nArrow keys or WASD to move\r\nPress Z to undo, press R to reset and press Q to quit\r\nMoves: {}\r\n\n{}",
                           map.name, render(&map), map.player.location_history.len(), help_text);
                } else {
                    let next_msg = if map_index >= maps.len() - 1 {
                        "Press Space to go back to the first level"
                    } else {
                        "Press Space to go to the next level"
                    };
                    print!("Level: {}\r\n{}\r\nYou did it!\r\n{}, press R to reset and press Q to quit\r\nMoves: {}\r\n\n{}",
                           map.name, render(&map), next_msg, map.player.location_history.len(), help_text);
                }
                stdout.flush()?;

                dirty = false
            }
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') if !map.all_objectives_met() => {
                            map.try_move_player('u');
                            dirty = true;
                        },
                        KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') if !map.all_objectives_met() => {
                            map.try_move_player('l');
                            dirty = true;
                        },
                        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') if !map.all_objectives_met() => {
                            map.try_move_player('d');
                            dirty = true;
                        },
                        KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') if !map.all_objectives_met() => {
                            map.try_move_player('r');
                            dirty = true;
                        },
                        KeyCode::Char('z') | KeyCode::Char('Z') if !map.all_objectives_met() => {
                            map.undo();
                            dirty = true;
                        },
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            map = maps[map_index].clone();
                            dirty = true;
                        },
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            return Ok(())
                        },
                        KeyCode::Char(' ') if map.all_objectives_met() => {
                            map_index += 1;
                            if map_index >= maps.len() {
                                map_index = 0;
                            }
                            map = maps[map_index].clone();
                            dirty = true;
                        },
                        KeyCode::Char('c')
                        if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                return Ok(())
                            }
                        _ => {}
                    }
                }
            }
        }
    })();

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    result
}
