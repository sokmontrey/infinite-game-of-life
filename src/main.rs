use std::collections::{HashMap, HashSet};
use std::net::IpAddr::V4;
use raylib::prelude::*;
use glam::IVec2;
use itertools::Itertools;

const SIMULATION_STEPS_PER_SECOND: u32 = 50;
const DRAG_THRESHOLD: i32 = 5;

fn main() {
    let (mut rl, thread) = init()
        .size(800, 800)
        .title("Infinite Conway's Game of Life")
        .build();

    let screen_width = rl.get_screen_width();
    let screen_height = rl.get_screen_height();

    let mut cell_size = 12;
    let mut origin = IVec2::ZERO;

    let mut cells: HashSet<IVec2> = HashSet::new();
    let mut is_dragging = false;
    let mut is_mouse_down = false;
    let mut mouse_down_pos = IVec2::ZERO;
    let mut previous_offset = IVec2::ZERO;

    let mut is_running = false;
    let mut last_time = rl.get_time();
    let mut last_frame_time = 0.0;

    while !rl.window_should_close() {
        let current_time = rl.get_time();
        let fps = rl.get_fps();

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        let current_mouse_pos = IVec2::new(
            d.get_mouse_position().x as i32,
            d.get_mouse_position().y as i32,
        );

        let dimensions = IVec2::new(screen_width, screen_height) / cell_size;
        let lower = -origin;
        let upper = lower + dimensions;

        // Cell rendering

        for cell in &cells {
            if cell.x < lower.x || cell.x >= upper.x || cell.y < lower.y || cell.y >= upper.y {
                continue;
            }

            let cell_screen_pos = cell + origin;
            d.draw_rectangle(
                cell_screen_pos.x * cell_size,
                cell_screen_pos.y * cell_size,
                cell_size,
                cell_size,
                Color::BLACK,
            );
        }

        // Board rendering

        for x in 0..=dimensions.x {
            let sx = x * cell_size;
            d.draw_line(sx, 0, sx, screen_height, Color::LIGHTGRAY);
        }
        for y in 0..=dimensions.y {
            let sy = y * cell_size;
            d.draw_line(0, sy, screen_width, sy, Color::LIGHTGRAY);
        }

        let hovered_cell = IVec2::new(
            ((current_mouse_pos.x - origin.x) as f32 / cell_size as f32).floor() as i32,
            ((current_mouse_pos.y - origin.y) as f32 / cell_size as f32).floor() as i32,
        );
        d.draw_rectangle_lines_ex(
            Rectangle::new(
                origin.x as f32 + (hovered_cell.x * cell_size) as f32,
                origin.y as f32 + (hovered_cell.y * cell_size) as f32,
                cell_size as f32,
                cell_size as f32,
            ),
            2.0,
            Color::RED,
        );

        // UI

        d.draw_text(&format!("FPS: {}", fps), 10, 10, 20, Color::GRAY);
        d.draw_text(&format!("Cells: {}", cells.len()), 10, 30, 20, Color::GRAY);
        d.draw_text(
            &format!("{}", if is_running { "Running" } else { "Paused" }),
            10,
            50,
            20,
            if is_running { Color::GREEN } else { Color::GRAY },
        );

        // Mouse input handling

        if d.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            let drag_distance: IVec2 = current_mouse_pos - mouse_down_pos;
            if !is_mouse_down {
                is_mouse_down = true;
                mouse_down_pos = current_mouse_pos;
                previous_offset = origin;
            } else if drag_distance.length_squared() > DRAG_THRESHOLD {
                is_dragging = true;
                let pan_delta = (current_mouse_pos - mouse_down_pos) / cell_size;
                origin = previous_offset + pan_delta;
            }
        } else if d.is_mouse_button_up(MouseButton::MOUSE_BUTTON_LEFT) && is_mouse_down {
            if !is_dragging {
                let cell = current_mouse_pos / cell_size - origin;

                if cells.contains(&cell) {
                    cells.remove(&cell);
                } else {
                    cells.insert(cell);
                }
            }

            is_dragging = false;
            is_mouse_down = false;
        }

        // Keyboard input handling

        if d.is_key_pressed(KeyboardKey::KEY_SPACE) {
            is_running = !is_running;
        } else if d.is_key_pressed(KeyboardKey::KEY_C) {
            cells.clear();
        }

        // Wheel input handling

        let wheel_move = d.get_mouse_wheel_move();
        if wheel_move.abs() > 0.0 {
            let zoom_factor = 1.1f32.powf(wheel_move);
            let new_cell_size = ((cell_size as f32 * zoom_factor).round() as i32).clamp(2, 100);
            if new_cell_size != cell_size {
                let mouse_pos = IVec2::new(
                    d.get_mouse_position().x as i32,
                    d.get_mouse_position().y as i32,
                );
                let world_before = (mouse_pos / cell_size) - origin;
                let world_after = (mouse_pos / new_cell_size) - origin;
                origin += world_before - world_after;
                cell_size = new_cell_size;
            }
        }

        // Simulation Logic

        if is_running {
            let elapsed_time = current_time - last_time;
            last_time = current_time;
            last_frame_time += elapsed_time;
            if last_frame_time >= (1.0 / SIMULATION_STEPS_PER_SECOND as f32) as f64 {
                last_frame_time = 0.0;
                cells = process_cells(&cells);
            }
        }
    }
}

fn process_cells(survived_cells: &HashSet<IVec2>) -> HashSet<IVec2> {
    let neighbour_counts = convolve(survived_cells);

    let survivors = survived_cells
        .iter()
        .filter(|&cell| match neighbour_counts.get(cell) {
            Some(2) | Some(3) => true,
            _ => false,
        }).copied();

    let births = neighbour_counts
        .iter()
        .filter(|&(_, &count)| count == 3)
        .map(|(&cell, _)| cell);

    survivors.chain(births).collect()
}

fn convolve(survived_cells: &HashSet<IVec2>) -> HashMap<IVec2, usize> {
    let deltas = (-1..=1)
        .cartesian_product(-1..=1)
        .map(|(x, y)| IVec2::new(x, y))
        .filter(|&d| d != IVec2::ZERO)
        .collect_vec();

    survived_cells.iter()
        .flat_map(|&cell| deltas.iter().map(move |&delta| cell + delta))
        .counts()
}
