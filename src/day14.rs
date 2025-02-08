use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::fs;
use std::{thread, time::Duration};

use crate::util::{Coordinate, Term};

// const INPUT_FILE: &str = "inputs/day14.test";
const INPUT_FILE: &str = "inputs/day14.input";

pub fn solve() {
    println!("Part 1: {}", solve_p1());
    println!("Part 2: {}", solve_p2());
}

#[derive(Debug)]
struct Robot {
    position: Coordinate<i64>,
    velocity: Coordinate<i64>,
}

impl Robot {
    fn from_string(line: &str) -> Robot {
        let (p, v) = line.split_once(" ").unwrap();
        let p = p.split_once("=").unwrap().1.split_once(",").unwrap();
        let v = v.split_once("=").unwrap().1.split_once(",").unwrap();
        let position = Coordinate::new(p.0.parse().unwrap(), p.1.parse().unwrap());
        let velocity = Coordinate::new(v.0.parse().unwrap(), v.1.parse().unwrap());

        Robot { position, velocity }
    }

    fn step(&mut self, n: i64, map_corner: Option<Coordinate<i64>>) {
        self.position = self.position + self.velocity * n;
        if let Some(map_corner) = map_corner {
            self.position = self.position % map_corner;
            // Do it again, so we have a positive number
            self.position = self.position + map_corner;
            self.position = self.position % map_corner;
        }
    }
}

struct Robots {
    robots: Vec<Robot>,
    map_corner: Coordinate<i64>,
}

impl Robots {
    fn from_string(content: &str, map_corner: Coordinate<i64>) -> Robots {
        let mut robots = vec![];
        for line in content.lines() {
            robots.push(Robot::from_string(line));
        }

        Robots { robots, map_corner }
    }

    fn step(&mut self, n: i64) {
        self.robots
            .iter_mut()
            .for_each(|r| r.step(n, Some(self.map_corner)));
    }

    fn ascii_art(&self, double_width: bool) -> String {
        let height = self.map_corner.y as usize;
        let mut width = self.map_corner.x as usize;
        if double_width {
            width *= 2;
        }

        let mut bitmap: Vec<Vec<char>> = vec![vec![' '; width + 3]; height + 2];
        self.robots.iter().for_each(|r| {
            let x = r.position.x as usize + 1;
            let y = r.position.y as usize + 1;
            if !double_width {
                bitmap[y][x] = '█';
            } else {
                bitmap[y][2 * x] = '█';
                bitmap[y][2 * x - 1] = '█';
            }
        });

        for (i, row) in bitmap.iter_mut().enumerate() {
            if i == 0 || i == height + 1 {
                row.fill('=')
            }
            row[0] = '#';
            row[width + 1] = '#';
            row[width + 2] = '\n';
        }

        String::from_iter(bitmap.iter().flatten())
    }

    fn check_line(&self) -> bool {
        let rows = self.map_corner.y as usize;
        let cols = self.map_corner.x as usize;

        let mut grid: Vec<Vec<bool>> = vec![vec![false; cols]; rows];
        self.robots.iter().for_each(|r| {
            let x = r.position.x as usize;
            let y = r.position.y as usize;
            grid[y][x] = true;
        });

        let size = (0.15 * (self.map_corner.x as f64)) as usize;

        // Check for square outlines
        for top in 0..rows {
            for left in 0..(cols - size) {
                let bottom = top + size - 1;
                let right = left + size - 1;

                if bottom >= rows
                    || right >= cols
                    || (!grid[top][left..=right].iter().all(|&x| x)
                        && !grid[bottom][left..=right].iter().all(|&x| x))
                {
                    continue;
                }

                return true;
            }
        }

        false
    }
}

fn get_safety_factor(map_corner: Coordinate<i64>, positions: Vec<Coordinate<i64>>) -> usize {
    let center = map_corner / 2;
    let mut quadrants: [usize; 4] = [0, 0, 0, 0];
    for position in positions {
        if position.x == center.x || position.y == center.y {
            continue;
        }
        let q = match (position.x < center.x, position.y < center.y) {
            (true, false) => 3,
            (false, true) => 2,
            (false, false) => 1,
            (true, true) => 0,
        };
        quadrants[q] += 1;
    }

    quadrants[0] * quadrants[1] * quadrants[2] * quadrants[3]
}

pub fn solve_p1() -> usize {
    let content = fs::read_to_string(INPUT_FILE).expect("Could not read input file");
    let mut final_positions: Vec<Coordinate<i64>> = vec![];

    // let corner = Coordinate::new(11, 7);
    let corner = Coordinate::new(101, 103);
    for line in content.lines() {
        let mut robot = Robot::from_string(line);
        robot.step(100, Some(corner));
        final_positions.push(robot.position);
    }
    get_safety_factor(corner, final_positions)
}

pub fn solve_p2() -> i64 {
    // let corner = Coordinate::new(11, 7);
    let corner = Coordinate::new(101, 103);

    let content = fs::read_to_string(INPUT_FILE).expect("Could not read input file");
    let mut robots = Robots::from_string(content.as_str(), corner);

    // Visualizes the tree appearing
    let mut sleep_time = 100000000;
    let mut paused = false;
    let mut reverse = false;
    let mut skip_frames = 1;
    let mut term = Term::init();
    let mut frame = 0;
    loop {
        if event::poll(Duration::from_nanos(sleep_time)).unwrap() {
            if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char(' ') => paused = !paused,
                    KeyCode::Char('r') => reverse = !reverse,
                    KeyCode::PageUp => skip_frames *= 2,
                    KeyCode::PageDown => skip_frames = (skip_frames / 2).max(1),
                    KeyCode::Up => sleep_time = (sleep_time / 2).max(1),
                    KeyCode::Down => sleep_time *= 2,
                    KeyCode::Enter => return frame,
                    KeyCode::Left => {
                        frame -= 1;
                        robots.step(-1);
                    }
                    KeyCode::Right => {
                        frame += 1;
                        robots.step(1);
                    }
                    _ => {}
                }
            }
        }
        if !paused {
            let step = if reverse { -1 } else { 1 };
            frame += step;
            robots.step(step);
        }
        let line_detected = robots.check_line();
        paused |= line_detected;

        if frame % skip_frames != 0 && !paused {
            continue;
        }

        let fb = format!(
            "[Frame {:4} (+={})] \t\t [{}]{} \t\t\t {} \n{}",
            frame,
            skip_frames,
            if paused { "PAUSED" } else { "RUNNING" },
            if reverse { "(REVERSING)" } else { "" },
            if line_detected {
                "--------- DETECTED A LINE ----------"
            } else {
                ""
            },
            robots.ascii_art(true),
        );

        term.draw(&fb);
        thread::sleep(Duration::from_nanos(sleep_time));
    }

    -1
}
