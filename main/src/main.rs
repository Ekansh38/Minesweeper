use ::rand::prelude::*;
use macroquad::prelude::*;

#[macroquad::main("Minesweeper")]
async fn main() {
    // Defining some constants
    //

    let flag_image: Texture2D = load_texture("./assets/flag.png").await.unwrap();
    let mine_image: Texture2D = load_texture("./assets/mine.png").await.unwrap();

    let images = [flag_image.clone(), mine_image.clone()];

    let rows = 9.0;
    let cols = 10.0;

    let size = 100.0;
    let mines = 10;
    let margin = rows * size / 7.0;

    // Set the window size to fit the board
    request_new_screen_size(cols * size, (rows * size) + margin);

    // Create a new board
    let mut board = Board::new(rows, cols, size, mines);

    loop {
        clear_background(WHITE);

        board.update();
        board.draw(margin, images.clone());

        next_frame().await;
    }
}

fn draw_resized_image(texture: Texture2D, x: f32, y: f32, width: f32, height: f32) {
    draw_texture_ex(
        &texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(Vec2::new(width, height)),
            ..Default::default()
        },
    );
}

#[derive(Clone)]
struct Cell {
    size: f32,         // Width and height of the cell
    pos: (f32, f32),   // Top-left corner of the cell
    nearby_mines: i32, // Number of mines in the 8 adjacent cells

    is_flagged: bool,  // Whether the cell is flagged
    is_mine: bool,     // Whether the cell is a mine
    is_revealed: bool, // Whether the cell is revealed
}

impl Cell {
    fn new(size: f32, pos: (f32, f32)) -> Self {
        Cell {
            size,
            pos,
            nearby_mines: 0,
            is_flagged: false,
            is_mine: false,
            is_revealed: false,
        }
    }

    fn draw(&self, images: [Texture2D; 2]) {
        let mut color = WHITE;

        if self.is_revealed {
            color = GRAY;
        }

        if self.is_revealed && self.is_mine && !self.is_flagged {
            draw_resized_image(
                images[1].clone(),
                self.pos.0 * self.size,
                self.pos.1 * self.size,
                self.size,
                self.size,
            );
            return;
        }

        if self.is_flagged {
            draw_resized_image(
                images[0].clone(),
                self.pos.0 * self.size,
                self.pos.1 * self.size,
                self.size,
                self.size,
            );
            return;
        }

        draw_rectangle(
            self.pos.0 * self.size as f32,
            self.pos.1 * self.size as f32,
            self.size,
            self.size,
            color,
        );

        if self.nearby_mines != 0 {
            let text = self.nearby_mines.to_string();
            let font_size = 80.0;
            let text_dims = measure_text(&text, None, font_size as u16, 1.0);
            draw_text(
                &text,
                self.pos.0 * self.size + (self.size - text_dims.width) / 2.0,
                self.pos.1 * self.size + (self.size + text_dims.height) / 2.0,
                font_size,
                BLACK,
            );
        }
    }
}

struct Board {
    rows: f32,
    cols: f32,
    size: f32,
    mines: i32,
    board: Vec<Cell>,
    has_clicked: bool,
    lose: bool,
    win: bool,
}

impl Board {
    fn new(rows: f32, cols: f32, size: f32, mines: i32) -> Self {
        let mut board: Vec<Cell> = vec![];

        for row in 0..rows as i32 {
            for col in 0..cols as i32 {
                let cell = Cell::new(size, (col as f32, row as f32));
                board.push(cell)
            }
        }

        Board {
            rows,
            cols,
            size,
            mines,
            board,
            has_clicked: false,
            lose: false,
            win: false,
        }
    }

    fn reveal(&mut self, cell_x: i32, cell_y: i32) {
        if cell_x < 0 || cell_x >= self.cols as i32 || cell_y < 0 || cell_y >= self.rows as i32 {
            return;
        }

        let index = cell_y as usize * self.cols as usize + cell_x as usize;

        {
            let cell = &mut self.board[index];
            if cell.is_revealed || cell.is_flagged || cell.is_mine {
                return;
            }
            cell.is_revealed = true;
        }

        let mut nearby_mines = 0;
        let dirs = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        for dir in dirs.iter() {
            let nx = cell_x + dir.0;
            let ny = cell_y + dir.1;
            if nx >= 0 && nx < self.cols as i32 && ny >= 0 && ny < self.rows as i32 {
                let neighbor_index = ny * self.cols as i32 + nx;
                if self.board[neighbor_index as usize].is_mine {
                    nearby_mines += 1;
                }
            }
        }

        self.board[index].nearby_mines = nearby_mines;

        if nearby_mines == 0 {
            self.board[index].is_flagged = false;
            // Flood fill
            for dir in dirs.iter() {
                let nx = cell_x as i32 + dir.0;
                let ny = cell_y as i32 + dir.1;
                if let Some(index) = self
                    .board
                    .iter()
                    .position(|cell| cell.pos == (nx as f32, ny as f32))
                {
                    if !self.board[index].is_revealed {
                        self.reveal(
                            self.board[index].pos.0 as i32,
                            self.board[index].pos.1 as i32,
                        );
                    }
                }
            }
        }
    }

    fn update(&mut self) {
        if self.lose {
            return;
        }

        let mut num_of_revealed = 0;

        for cell in self.board.iter() {
            if cell.is_revealed && !cell.is_mine {
                num_of_revealed += 1;
            }
        }

        if num_of_revealed == (self.rows * self.cols - self.mines as f32) as usize {
            self.win()
        }

        self.check_input();
    }

    fn win(&mut self) {
        self.win = true;
        for cell in self.board.iter_mut() {
            if cell.is_mine {
                cell.is_flagged = true;
            }

            cell.is_revealed = true;
        }
    }
    fn place_mines(&mut self, mouse_pos: (f32, f32)) {
        self.has_clicked = true;
        let mut mines_placed = 0;

        let mut rng = ::rand::thread_rng();

        loop {
            if mines_placed >= self.mines {
                break;
            }

            let random_index = rng.gen_range(0..self.board.len());
            let cell = &mut self.board[random_index];

            if cell.is_mine {
                continue;
            } else if cell.pos != mouse_pos {
                mines_placed += 1;
                cell.is_mine = true;
            }
        }
    }

    fn game_over(&mut self) {
        self.lose = true;
        for cell in self.board.iter_mut() {
            if cell.is_mine {
                cell.is_revealed = true;
            }

            if cell.is_flagged && !cell.is_mine {
                cell.is_revealed = true;
                cell.is_flagged = false;
            }
        }
    }

    fn check_input(&mut self) {
        let mouse_pos = mouse_position();
        let mouse_x = mouse_pos.0 as i32 / self.size as i32;
        let mouse_y = mouse_pos.1 as i32 / self.size as i32;
        let mouse_pos = (mouse_x as f32, mouse_y as f32);

        if mouse_x >= 0 && mouse_x < self.cols as i32 && mouse_y >= 0 && mouse_y < self.rows as i32
        {
            if is_mouse_button_pressed(MouseButton::Left) {
                // Place mines on first click
                if !self.has_clicked {
                    self.place_mines(mouse_pos);
                }

                if let Some(index) = self.board.iter().position(|cell| cell.pos == mouse_pos) {
                    let clicked_cell = &mut self.board[index];
                    if !clicked_cell.is_revealed {
                        if clicked_cell.is_mine {
                            self.game_over();
                            return;
                        } else {
                            self.reveal(mouse_x, mouse_y);
                        }
                    }
                }
            } else if is_mouse_button_pressed(MouseButton::Right) {
                if let Some(index) = self.board.iter().position(|cell| cell.pos == mouse_pos) {
                    let clicked_cell = &mut self.board[index];
                    if !clicked_cell.is_revealed {
                        clicked_cell.is_flagged = !clicked_cell.is_flagged;
                    }
                }
            }
        }
    }

    fn draw(&self, margin: f32, images: [Texture2D; 2]) {
        for cell in self.board.iter() {
            cell.draw(images.clone());
        }

        for row in 0..self.rows as i32 + 1 {
            draw_line(
                0.0,
                row as f32 * self.size,
                self.cols * self.size,
                row as f32 * self.size,
                2.0,
                BLACK,
            );
        }

        for col in 0..self.cols as i32 + 1 {
            draw_line(
                col as f32 * self.size,
                0.0,
                col as f32 * self.size,
                self.rows * self.size,
                2.0,
                BLACK,
            );
        }

        // Draw how many mines are left
        let mines_left =
            self.mines - self.board.iter().filter(|cell| cell.is_flagged).count() as i32;

        let mut text = String::new();
        if self.lose {
            text = "Game Over!".to_string();
        } else if self.win {
            text = "You Win!".to_string();
        } else {
            text = format!("Mines left: {}", mines_left);
        }

        let font_size = 40.0;
        let text_dims = measure_text(&text, None, font_size as u16, 1.0);
        let x = self.cols * self.size / 2.0 - text_dims.width / 2.0;
        let y = self.rows * self.size + margin / 2.0 + text_dims.height / 2.0;
        draw_text(&text, x, y, font_size, BLACK);
    }
}
