use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::{stdin, stdout, Write};

use crate::configuration;
struct Game {
    game_board: Vec<Vec<char>>,
    player_turn: u8,
    player_symbols: Vec<char>,
    player_symbol: char,
    max_input_y: usize,
    max_input_x: usize,
    last_valid_move: (usize, usize, char),
}
pub fn run(config: configuration::Config) -> Result<String, u8> {
    let mut game = Game::build(config);
    game.print();

    let mut game_over: bool = false;
    let mut change_player;
    while !game_over {
        let mut valid_move: bool = false;
        while !valid_move {
            print!("{}> ", game.player_symbol);
            valid_move = game.read_player_move()?;
        }
        println!();

        // Update the game board with player symbol if a box is complete
        change_player = game.update_game_board();

        // Check game over condition
        game_over = game.check_game_over();

        //Print the game board
        game.print();

        if !game_over && change_player {
            // Change player turn
            game.player_turn += 1;
            if game.player_turn > game.player_symbols.len() as u8 {
                game.player_turn = 1;
            }
            game.player_symbol = game.player_symbols[(game.player_turn - 1) as usize];
        }
    }

    //Determine winners
    Ok(game.determine_winners())
}

impl Game {
    fn print(&self) {
        for row in &self.game_board {
            for c in row {
                print!("{c}");
            }
            println!();
        }
    }

    fn build(config: configuration::Config) -> Game {
        let mut game_board: Vec<Vec<char>> = Vec::new();

        let expanded_height = config.height * 2 - 1;
        let expanded_width = config.width * 2 - 1;

        // Generate a default board
        for row_index in 0..(expanded_height) {
            let mut game_row: Vec<char> = Vec::new();
            if row_index % 2 == 0 {
                for column_index in 0..(expanded_width) {
                    if column_index % 2 == 0 {
                        game_row.push('+');
                    } else {
                        game_row.push(' ');
                    }
                }
            } else {
                for _column_index in 0..(expanded_width) {
                    game_row.push(' ');
                }
            }
            game_board.push(game_row);
        }

        // Generate maximum input sizes so the calculation doesn't have to always re-evaluate
        let max_input_y = game_board.len() - ((game_board.len() - 1) / 2) - 1;
        let max_input_x: usize = game_board[0].len() - ((game_board[0].len() - 1) / 2) - 1;

        // Fill board with data from save if it exists
        if !config.board_edges.is_empty() && !config.board_cells.is_empty() {
            // Fill edges
            let mut x_fill_index: usize;
            let mut fill_char: char;
            for (y_fill_index, horizontal) in config.board_edges.iter().enumerate() {
                if y_fill_index % 2 == 0 {
                    x_fill_index = 1;
                    fill_char = '-';
                } else {
                    x_fill_index = 0;
                    fill_char = '|';
                }
                for c in horizontal.chars() {
                    if c == '1' {
                        game_board[y_fill_index][x_fill_index] = fill_char;
                    }
                    x_fill_index += 2;
                }
            }

            // Fill cells
            let mut y_fill_index: usize = 1;
            let mut x_fill_index: usize = 1;
            for horizontal in &config.board_cells {
                for c in horizontal.chars() {
                    if c.is_numeric() {
                        if c != '0' {
                            game_board[y_fill_index][x_fill_index] =
                                (c.to_digit(10).unwrap() as u8 + 64) as char;
                        }
                        x_fill_index += 2;
                    }
                }
                y_fill_index += 2;
                x_fill_index = 1;
            }
        }

        // Generate player data
        let player_turn = config.player_turn;
        let mut player_symbols: Vec<char> = Vec::new();
        for symbol in 1..config.player_count + 1 {
            player_symbols.push((symbol + 64) as char);
        }
        let player_symbol = player_symbols[(player_turn - 1) as usize];

        Game {
            game_board,
            player_turn,
            player_symbols,
            player_symbol,
            max_input_y,
            max_input_x,
            last_valid_move: (0, 0, 'x'),
        }
    }

    fn read_player_move(&mut self) -> Result<bool, u8> {
        let mut player_move: String = String::new();
        let _ = stdout().flush();

        match stdin().read_line(&mut player_move) {
            Ok(test) => test,
            Err(_) => return Err(6),
        };

        // Trim CR and LF from input and format into vector of strings
        player_move = player_move.replace(['\n', '\r'], "");
        let player_move: Vec<&str> = player_move.split(' ').collect();

        if player_move.len() == 2 && player_move[0] == "w" {
            Ok(self.save_game(player_move[1])?)
        } else {
            Ok(self.validate_player_move(player_move))
        }
    }

    fn validate_player_move(&mut self, player_move: Vec<&str>) -> bool {
        // Ensure only 3 arguments are given
        if player_move.len() != 3 {
            return false;
        }
        // Parse input coordinantes as usize. Automatically rejects negative input
        let Ok(move_y) = player_move[0].parse::<usize>() else {
            return false;
        };
        let Ok(move_x) = player_move[1].parse::<usize>() else {
            return false;
        };

        // Check the player edge option
        if player_move[2].len() != 1 {
            return false;
        }
        let Some(player_edge_option) = player_move[2].chars().next() else {
            return false;
        };

        let translated_y: usize;
        let translated_x: usize;
        // Process horizontal moves
        if player_edge_option == 'h' {
            if (move_y > self.max_input_y || move_x >= self.max_input_x)
                || self.game_board[move_y * 2][(move_x * 2) + 1] != ' '
            {
                return false;
            } else {
                translated_y = move_y * 2;
                translated_x = (move_x * 2) + 1;
                self.game_board[translated_y][translated_x] = '-';
            }
            // Process vertival moves
        } else if player_edge_option == 'v' {
            if (move_y >= self.max_input_y || move_x > self.max_input_x)
                || self.game_board[(move_y * 2) + 1][move_x * 2] != ' '
            {
                return false;
            } else {
                translated_y = (move_y * 2) + 1;
                translated_x = move_x * 2;
                self.game_board[translated_y][translated_x] = '|';
            }
        } else {
            return false;
        }

        // Save the move for processing later
        self.last_valid_move = (translated_y, translated_x, player_edge_option);

        true
    }

    fn update_game_board(&mut self) -> bool {
        let mut change_player = true;
        let player_symbol = self.player_symbols[(self.player_turn - 1) as usize];

        if self.last_valid_move.2 == 'h' {
            // Check above
            if self.last_valid_move.0 > 0 {
                let above_index_y = self.last_valid_move.0 - 1;
                if self.game_board[above_index_y - 1][self.last_valid_move.1] == '-'
                    && self.game_board[above_index_y][self.last_valid_move.1 - 1] == '|'
                    && self.game_board[above_index_y][self.last_valid_move.1 + 1] == '|'
                {
                    self.game_board[above_index_y][self.last_valid_move.1] = player_symbol;
                    change_player = false;
                }
            }
            // Check below
            if self.last_valid_move.0 < self.game_board.len() - 1 {
                let below_index_y = self.last_valid_move.0 + 1;
                if self.game_board[below_index_y + 1][self.last_valid_move.1] == '-'
                    && self.game_board[below_index_y][self.last_valid_move.1 - 1] == '|'
                    && self.game_board[below_index_y][self.last_valid_move.1 + 1] == '|'
                {
                    self.game_board[below_index_y][self.last_valid_move.1] = player_symbol;
                    change_player = false;
                }
            }
        } else if self.last_valid_move.2 == 'v' {
            // Check left
            if self.last_valid_move.1 > 0 {
                let left_index_x = self.last_valid_move.1 - 1;
                if self.game_board[self.last_valid_move.0][left_index_x - 1] == '|'
                    && self.game_board[self.last_valid_move.0 - 1][left_index_x] == '-'
                    && self.game_board[self.last_valid_move.0 + 1][left_index_x] == '-'
                {
                    self.game_board[self.last_valid_move.0][left_index_x] = player_symbol;
                    change_player = false;
                }
            }
            // Check right
            if self.last_valid_move.1 < self.game_board[0].len() - 1 {
                let right_index_x = self.last_valid_move.1 + 1;
                if self.game_board[self.last_valid_move.0][right_index_x + 1] == '|'
                    && self.game_board[self.last_valid_move.0 - 1][right_index_x] == '-'
                    && self.game_board[self.last_valid_move.0 + 1][right_index_x] == '-'
                {
                    self.game_board[self.last_valid_move.0][right_index_x] = player_symbol;
                    change_player = false;
                }
            }
        }

        change_player
    }

    fn check_game_over(&mut self) -> bool {
        let mut game_over: bool = true;
        for (y_index, row) in self.game_board.iter().enumerate().skip(1) {
            // Iterate every second row
            if y_index % 2 != 0 {
                for (x_index, c) in row.iter().enumerate().skip(1) {
                    if x_index % 2 != 0
                        && (self.game_board[y_index][x_index - 1] != '|'
                            || self.game_board[y_index][x_index + 1] != '|'
                            || self.game_board[y_index - 1][x_index] != '-'
                            || self.game_board[y_index + 1][x_index] != '-')
                    {
                        game_over = false;
                    }
                }
            }
        }

        game_over
    }

    fn determine_winners(self) -> String {
        // Generate hash map of player scores
        let mut winners_map: HashMap<char, u16> = HashMap::new();
        for (row_index, row) in self.game_board.iter().enumerate() {
            if row_index % 2 != 0 {
                for (column_index, c) in row.iter().enumerate() {
                    if column_index % 2 != 0 {
                        // Insert the key for found player or increment if it exists.
                        *winners_map.entry(*c).or_insert(0) += 1;
                    }
                }
            }
        }

        // Generate string of winners
        let mut winners = String::new();
        let mut highest_score = 0;
        for (key, value) in winners_map.iter() {
            match value.cmp(&highest_score) {
                Ordering::Greater => {
                    highest_score = *value;
                    winners.clear();
                    winners.push(*key);
                }
                Ordering::Equal => {
                    winners.push_str(", ");
                    winners.push(*key);
                }
                Ordering::Less => (),
            };
        }

        winners
    }

    fn save_game(&self, path: &str) -> Result<bool, u8> {
        let mut edges = String::new();
        let mut cells: String = String::new();
        let mut add_comma: bool = false;

        for (iy, row) in self.game_board.iter().enumerate() {
            for (ix, c) in row.iter().enumerate() {
                // Process edge data
                if ((iy % 2 == 0 && ix % 2 != 0) || (iy % 2 != 0 && ix % 2 == 0)) && *c != ' ' {
                    edges.push('1');
                } else if ((iy % 2 == 0 && ix % 2 != 0) || (iy % 2 != 0 && ix % 2 == 0))
                    && *c == ' '
                {
                    edges.push('0');
                }

                // Process cell data
                if iy % 2 != 0 && ix % 2 != 0 && *c == ' ' {
                    if add_comma {
                        cells.push(',');
                    }
                    add_comma = true;
                    cells.push('0');
                } else if iy % 2 != 0 && ix % 2 != 0 && *c != ' ' {
                    if add_comma {
                        cells.push(',');
                    }
                    add_comma = true;
                    let Some(char_save_value) = char::from_digit((*c as u32) - 64, 10) else {
                        return Err(9);
                    };
                    cells.push(char_save_value);
                }
            }
            add_comma = false;

            edges.push('\n');
            if iy % 2 != 0 {
                cells.push('\n');
            }
        }

        // Construct single string with newlines to avoid multiple IO
        let mut save_contents = self.player_turn.to_string();
        save_contents.push('\n');
        save_contents.push_str(&edges);
        save_contents.push_str(&cells);

        let Ok(_file) = OpenOptions::new().write(true).create_new(true).open(path) else {
            eprintln!("Error opening file for saving grid");
            return Ok(false);
        };
        let Ok(_result) = fs::write(path, save_contents) else {
            return Err(9);
        };
        eprintln!("Save of grid successful");

        Ok(false)
    }
}
