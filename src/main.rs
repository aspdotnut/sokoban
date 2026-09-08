use clap::Parser;
use crossterm::{
    cursor,
    event,
    event::{Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal,
    terminal::{ClearType},
    style::{Stylize}
};
use std::{fs,
          io::{stdout, Write},
          path::{PathBuf},
          time::{Duration}
};

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
    moves: i32,
    location_history: Vec<(i32, i32)>
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
    y: i32
}

#[derive(Clone)]
struct Finish {
    x: i32,
    y: i32
}

#[derive(Clone)]
struct Wall {
    x: i32,
    y: i32
}

impl Map {
    fn try_move_player(&mut self, direction: char) {
        let mut new_x = self.player.x;
        let mut new_y = self.player.y;

        match direction {
            'u' => new_y = self.player.y - 1,
            'd' => new_y = self.player.y + 1,
            'l' => new_x = self.player.x - 1,
            'r' => new_x = self.player.x + 1,
            _ => {}
        }

        self.player.location_history.push((self.player.x, self.player.y));

        for cube in self.cubes.iter_mut() {
            cube.location_history.push((cube.x, cube.y));
        }

        if new_x < 0 || new_x >= self.width ||
            new_y < 0 || new_y >= self.height {
            return;
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

        self.player.x = new_x;
        self.player.y = new_y;
        self.player.moves += 1;
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
            return false;
        }

        for (i, other_cube) in self.cubes.iter().enumerate() {
            if i != index &&
                other_cube.x == new_x &&
                other_cube.y == new_y {
                return false;
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

            if self.player.moves > 0 {
                self.player.moves -= 1
            }
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
        if (self.player.x != self.finish.x || self.player.y != self.finish.y) && self.finish.x != 0 && self.finish.y != 0 {
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
}

fn load_maps(filepath: &PathBuf) -> Vec<Map> {
    let contents = fs::read_to_string(filepath)
        .expect("Failed to read file");

    contents
        .split(';')
        .filter_map(|level| {
            let mut lines = level.lines();

            let header = lines.find(|line| !line.trim().is_empty())?;

            let map_lines: Vec<&str> = lines
                .filter(|line| !line.trim().is_empty())
                .collect();

            if map_lines.is_empty() {
                return None;
            }

            Some(parse_map(header, &map_lines))
        })
        .collect()
}

fn parse_map(header: &str, lines: &[&str]) -> Map {
    let name = header.trim().to_string();

    let height = lines.len() as i32;
    let width = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0) as i32;

    let mut player = Player {
        x: 0,
        y: 0,
        moves: 0,
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
                'F' => {
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
            out.push_str(&ch);
        }
        out.push('\r');
        out.push('\n');
    }
    out
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let filepath = match args.file {
        Some(file) => PathBuf::from(file),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/microban.txt"),
    };
    let maps = load_maps(&filepath);

    let mut map_index = 0;

    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    let mut map = maps[map_index].clone();

    let result = (|| -> std::io::Result<()> {
        loop {
            if !map.all_objectives_met() {
                execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                print!("Level: {}", map.name);
                print!("\r\n{}", render(&map));
                print!("\r\nArrow keys or WASD to move");
                print!("\r\nPress z to undo, press r to reset and press q to quit");
                print!("\r\nMoves: {}", map.player.moves);
                stdout.flush()?;

                if event::poll(Duration::from_millis(200))? {
                    if let Event::Key(key_event) = event::read()? {
                        if key_event.kind == KeyEventKind::Press {
                            match key_event.code {
                                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W')  => map.try_move_player('u'),
                                KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => map.try_move_player('l'),
                                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => map.try_move_player('d'),
                                KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => map.try_move_player('r'),
                                KeyCode::Char('z') | KeyCode::Char('Z') => map.undo() ,
                                KeyCode::Char('r') | KeyCode::Char('R') => { map = maps[map_index].clone(); },
                                KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                                KeyCode::Char('c')
                                    if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                                    {
                                        return Ok(());
                                    }
                                _ => {}
                            }
                        }
                    }
                }
            } else {
                execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                print!("Level: {}", map_index + 1);
                print!("\r\n{}", render(&map));
                print!("\r\nYou did it!");
                if map_index >= maps.len() - 1 {
                    print!("\r\nPress space to go back to the first level, press r to reset and press q to quit");
                } else {
                    print!("\r\nPress space to go to the next level, press r to reset and press q to quit");
                }
                print!("\r\nMoves: {}", map.player.moves);
                stdout.flush()?;

                if event::poll(Duration::from_millis(200))? {
                    if let Event::Key(key_event) = event::read()? {
                        if key_event.kind == KeyEventKind::Press {
                            match key_event.code {
                                KeyCode::Char(' ') => {
                                    map_index += 1;
                                    if map_index >= maps.len() {
                                        map_index = 0;
                                    }
                                    map = maps[map_index].clone();
                                },
                                KeyCode::Char('r') | KeyCode::Char('R') => { map = maps[map_index].clone(); },
                                KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                                KeyCode::Char('c')
                                    if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                                    {
                                        return Ok(());
                                    }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    })();

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    result

}
