use crossterm::{
    cursor,
    event,
    event::{Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal,
    terminal::{ClearType},
    style::{Stylize}
};
use std::io::{stdout, Write};
use std::time::Duration;

#[derive(Clone)]
struct Map {
    width: i32,
    height: i32,
    par: i32,
    player: Player,
    cubes: Vec<Cube>,
    buttons: Vec<Button>,
    finish: Finish, // note to self: disable by setting to 0, 0
    walls: Vec<Wall>,
}

#[derive(Clone)]
struct Player {
    x: i32,
    y: i32,
    moves: i32,
    backtrack: Vec<(i32, i32)>
}

#[derive(Clone)]
struct Cube {
    x: i32,
    y: i32,
    backtrack: Vec<(i32, i32)>,
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

        self.player.backtrack.push((self.player.x, self.player.y));

        for cube in self.cubes.iter_mut() {
            cube.backtrack.push((cube.x, cube.y));
        }

        if new_x < 1 || new_x > self.width ||
            new_y < 1 || new_y > self.height {
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

        if new_x < 1 || new_x > self.width ||
            new_y < 1 || new_y > self.height {
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

    fn backtrack(&mut self) {
        if let Some((x, y)) = self.player.backtrack.pop() {
            self.player.x = x;
            self.player.y = y;

            if self.player.moves > 0 {
                self.player.moves -= 1
            }
        }

        for cube in self.cubes.iter_mut() {
            if let Some((x, y)) = cube.backtrack.pop() {
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

fn render(map: &Map) -> String {
    let mut out = String::new();

    for y in 0..map.height + 2 {
        for x in 0..map.width + 2 {
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
                wall.x == x && wall.y == y || x == 0 || y == 0 || x == map.width + 1 || y == map.height + 1
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
    let mut map_index = 0;
    let maps: Vec<Map> = vec![
        Map {
            width: 14,
            height: 7,
            par: 84,
            player: Player { x: 1, y: 2, moves: 0, backtrack: Vec::new() },
            cubes: vec![
                Cube { x: 6, y: 3, backtrack: Vec::new() },
                Cube { x: 13, y: 2, backtrack: Vec::new() },
            ],
            buttons: vec![
                Button { x: 2, y: 7 },
                Button { x: 5, y: 7 },
            ],
            finish: Finish { x: 0, y: 0 },
            walls: vec![
                Wall { x: 1, y: 1 },
                Wall { x: 2, y: 1 },
                Wall { x: 3, y: 1 },
                Wall { x: 7, y: 1 },
                Wall { x: 8, y: 1 },
                Wall { x: 9, y: 1 },
                Wall { x: 10, y: 1 },
                Wall { x: 11, y: 1 },
                Wall { x: 12, y: 1 },
                Wall { x: 13, y: 1 },
                Wall { x: 14, y: 1 },
                Wall { x: 1, y: 4 },
                Wall { x: 2, y: 4 },
                Wall { x: 3, y: 4 },
                Wall { x: 4, y: 4 },
                Wall { x: 5, y: 4 },
                Wall { x: 6, y: 4 },
                Wall { x: 7, y: 4 },
                Wall { x: 8, y: 4 },
                Wall { x: 9, y: 4 },
                Wall { x: 10, y: 4 },
                Wall { x: 13, y: 4 },
                Wall { x: 14, y: 4 },
                Wall { x: 1, y: 5 },
                Wall { x: 2, y: 5 },
                Wall { x: 3, y: 5 },
                Wall { x: 4, y: 5 },
                Wall { x: 8, y: 5 },
                Wall { x: 9, y: 5 },
                Wall { x: 10, y: 5 },
                Wall { x: 13, y: 5 },
                Wall { x: 14, y: 5 },
                Wall { x: 8, y: 7 },
                Wall { x: 9, y: 7 },
                Wall { x: 10, y: 7 },
                Wall { x: 11, y: 7 },
                Wall { x: 12, y: 7 },
                Wall { x: 13, y: 7 },
                Wall { x: 14, y: 7 },
            ],
        },
        Map {
            width: 20,
            height: 11,
            par: 117,
            player: Player { x: 1, y: 11, moves: 0, backtrack: Vec::new() },
            cubes: vec![
                Cube { x: 7, y: 3, backtrack: Vec::new() },
                Cube { x: 9, y: 7, backtrack: Vec::new() },
                Cube { x: 19, y: 11, backtrack: Vec::new() },
            ],
            buttons: vec![
                Button { x: 1, y: 3, },
                Button { x: 8, y: 7, },
                Button { x: 18, y: 11, },
            ],
            finish: Finish { x: 2, y: 1 },
            walls: vec![
                Wall { x: 1, y: 1, },
                Wall { x: 11, y: 1, },
                Wall { x: 12, y: 1, },
                Wall { x: 1, y: 2, },
                Wall { x: 2, y: 2, },
                Wall { x: 3, y: 2, },
                Wall { x: 4, y: 2, },
                Wall { x: 5, y: 2, },
                Wall { x: 6, y: 2, },
                Wall { x: 7, y: 2, },
                Wall { x: 8, y: 2, },
                Wall { x: 9, y: 2, },
                Wall { x: 11, y: 2, },
                Wall { x: 12, y: 2, },
                Wall { x: 14, y: 2, },
                Wall { x: 15, y: 2, },
                Wall { x: 16, y: 2, },
                Wall { x: 17, y: 2, },
                Wall { x: 18, y: 2, },
                Wall { x: 19, y: 2, },
                Wall { x: 9, y: 3, },
                Wall { x: 14, y: 3, },
                Wall { x: 1, y: 4, },
                Wall { x: 2, y: 4, },
                Wall { x: 3, y: 4, },
                Wall { x: 4, y: 4, },
                Wall { x: 5, y: 4, },
                Wall { x: 6, y: 4, },
                Wall { x: 7, y: 4, },
                Wall { x: 9, y: 4, },
                Wall { x: 10, y: 4, },
                Wall { x: 11, y: 4, },
                Wall { x: 12, y: 4, },
                Wall { x: 13, y: 4, },
                Wall { x: 14, y: 4, },
                Wall { x: 16, y: 4, },
                Wall { x: 17, y: 4, },
                Wall { x: 18, y: 4, },
                Wall { x: 19, y: 4, },
                Wall { x: 20, y: 4, },
                Wall { x: 5, y: 5, },
                Wall { x: 14, y: 5, },
                Wall { x: 1, y: 6, },
                Wall { x: 2, y: 6, },
                Wall { x: 3, y: 6, },
                Wall { x: 5, y: 6, },
                Wall { x: 7, y: 6, },
                Wall { x: 8, y: 6, },
                Wall { x: 9, y: 6, },
                Wall { x: 11, y: 6, },
                Wall { x: 13, y: 6, },
                Wall { x: 14, y: 6, },
                Wall { x: 15, y: 6, },
                Wall { x: 16, y: 6, },
                Wall { x: 17, y: 6, },
                Wall { x: 18, y: 6, },
                Wall { x: 20, y: 6, },
                Wall { x: 5, y: 7, },
                Wall { x: 7, y: 7, },
                Wall { x: 11, y: 7, },
                Wall { x: 13, y: 7, },
                Wall { x: 2, y: 8, },
                Wall { x: 3, y: 8, },
                Wall { x: 4, y: 8, },
                Wall { x: 5, y: 8, },
                Wall { x: 7, y: 8, },
                Wall { x: 8, y: 8, },
                Wall { x: 9, y: 8, },
                Wall { x: 10, y: 8, },
                Wall { x: 11, y: 8, },
                Wall { x: 12, y: 8, },
                Wall { x: 13, y: 8, },
                Wall { x: 14, y: 8, },
                Wall { x: 15, y: 8, },
                Wall { x: 17, y: 8, },
                Wall { x: 18, y: 8, },
                Wall { x: 19, y: 8, },
                Wall { x: 20, y: 8, },
                Wall { x: 2, y: 9, },
                Wall { x: 4, y: 9, },
                Wall { x: 10, y: 9, },
                Wall { x: 2, y: 10, },
                Wall { x: 4, y: 10, },
                Wall { x: 5, y: 10, },
                Wall { x: 7, y: 10, },
                Wall { x: 9, y: 10, },
                Wall { x: 10, y: 10, },
                Wall { x: 11, y: 10, },
                Wall { x: 12, y: 10, },
                Wall { x: 13, y: 10, },
                Wall { x: 14, y: 10, },
                Wall { x: 16, y: 10, },
                Wall { x: 17, y: 10, },
                Wall { x: 18, y: 10, },
                Wall { x: 19, y: 10, },
                Wall { x: 7, y: 11, },
                Wall { x: 17, y: 11, },
            ],
        },
    ];

    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    let mut map = maps[map_index].clone();

    let result = (|| -> std::io::Result<()> {
        loop {
            if !map.all_objectives_met() {
                execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                print!("Level: {}", map_index + 1);
                print!("\r\n{}", render(&map));
                print!("\r\nArrow keys or WASD to move");
                print!("\r\nPress b to backtrack, press r to reset and press q to quit");
                print!("\r\nPar: {}, Moves: {}", map.par, map.player.moves);
                // string for testing, disabling it but won't delete yet
                // print!("\r\nPlayer x: {}, Player y: {}", map.player.x, map.player.y);
                stdout.flush()?;

                if event::poll(Duration::from_millis(200))? {
                    if let Event::Key(key_event) = event::read()? {
                        if key_event.kind == KeyEventKind::Press {
                            match key_event.code {
                                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W')  => map.try_move_player('u'),
                                KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => map.try_move_player('l'),
                                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => map.try_move_player('d'),
                                KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => map.try_move_player('r'),
                                KeyCode::Char('b') | KeyCode::Char('B') => map.backtrack() ,
                                KeyCode::Char('r') | KeyCode::Char('R') => { map = maps[map_index].clone(); },
                                KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                                KeyCode::Char('c')
                                    if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                                    {
                                        return Ok(());
                                    }
                                KeyCode::Char('n') | KeyCode::Char('N') => {
                                    map_index += 1;
                                    if map_index >= maps.len() {
                                        map_index = 0;
                                    }
                                    map = maps[map_index].clone();
                                },
                                _ => {}
                            }
                        }
                    }
                }
            } else {
                execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                print!("Level: {}", map_index + 1);
                print!("\r\n{}", render(&map));
                print!("\r\nPress n to go to the next level, press r to reset and press q to quit");
                print!("\r\nPar: {}, Moves: {}", map.par, map.player.moves);
                let move_delta = map.par - map.player.moves;
                if move_delta > 0 {
                    print!("\r\n{} below par", move_delta);
                } else if move_delta == 0 {
                    print!("\r\nPar");
                } else {
                    print!("\r\n{} above par", move_delta.to_string().trim_matches('-'));
                }
                print!("\r\nYou won!");
                stdout.flush()?;

                if event::poll(Duration::from_millis(200))? {
                    if let Event::Key(key_event) = event::read()? {
                        if key_event.kind == KeyEventKind::Press {
                            match key_event.code {
                                KeyCode::Char('n') | KeyCode::Char('N') => {
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
