use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Range;

#[derive(Debug)]
pub struct Config {
    pub height: u16,
    pub width: u16,
    pub player_count: u8,
    pub player_turn: u8,
    pub board_edges: Vec<String>,
    pub board_cells: Vec<String>,
}

impl Config {
    pub fn build(args: impl Iterator<Item = String>) -> Result<Config, u8> {
        // Convert args to vector of strings
        let options: Vec<String> = args.skip(1).collect();

        // Check argument length
        if options.len() != 3 && options.len() != 4 {
            println!("options.len {}", options.len());
            return Err(1);
        }

        // Parse height argument
        let Ok(height) = options[0].parse::<u16>() else {
            return Err(2);
        };

        // Parse width argument
        let Ok(width) = options[1].parse::<u16>() else {
            return Err(2);
        };

        // Ensure height and width are within valid range
        let valid_board_dimensions: Range<u16> = 2..1000;
        if !valid_board_dimensions.contains(&height) || !valid_board_dimensions.contains(&width) {
            return Err(2);
        };

        // Parse player_count argument
        let Ok(player_count) = options[2].parse::<u8>() else {
            return Err(3);
        };

        // Validate number of players - Minimum of 2, only allow chars up to 'Z'
        let valid_player_count: Range<u8> = 2..101;
        if !valid_player_count.contains(&player_count) {
            return Err(3);
        };

        // Initialise struct
        let mut config: Config = Config {
            height,
            width,
            player_count,
            player_turn: 1,
            board_edges: Vec::new(),
            board_cells: Vec::new(),
        };

        // Return early if save game wasn't given
        if options.len() == 3 {
            Ok(config)
        } else {
            let Ok(file_content) = File::open(&options[3]) else {
                return Err(4);
            };

            // Attempt to read lines into a vector of strings
            let Ok(lines) = BufReader::new(&file_content)
                .lines()
                .collect::<Result<Vec<String>, _>>() else {
                return Err(5);
            };

            // Parse player turn
            let Ok(player_turn) = lines[0].parse::<u8>() else {
                return Err(5);
            };
            let valid_player_turns: Range<u8> = 1..101;
            if !valid_player_turns.contains(&player_turn) {
                return Err(5);
            }
            config.player_turn = player_turn;

            // Parse board edges and cells
            for line in lines.iter().skip(1) {
                if !line.contains(',') {
                    config.board_edges.push(line.to_string());
                } else {
                    config.board_cells.push(line.to_string());
                }
            }

            // Validate the save file contents
            if !valid_edge_data(&config.board_edges, config.height, config.width)
                || !valid_cell_data(
                    &config.board_cells,
                    config.height,
                    config.width,
                    config.player_count,
                )
            {
                return Err(5);
            }

            Ok(config)
        }
    }
}

fn valid_edge_data(edge_data: &[String], height: u16, width: u16) -> bool {
    let mut row_data_count: u16 = 0;
    let mut col_data_count: u16 = 0;

    // Validate edge position data against game board
    for (row, data) in edge_data.iter().enumerate() {
        // From idx 0, every second line should be of size width - 1
        if row % 2 == 0 {
            row_data_count += 1;
            if width as usize - 1 != data.len() {
                return false;
            }
        // From idx 1, every second line should be of size width
        } else {
            col_data_count += 1;
            if width as usize != data.len() {
                return false;
            }
        }
    }

    // Validate amount of row and column data against game board
    if row_data_count != height || col_data_count != width - 1 {
        return false;
    }

    true
}

fn valid_cell_data(cell_data: &[String], height: u16, width: u16, player_count: u8) -> bool {
    let mut rows: u16 = 0;
    let valid_cell_entries: Range<u8> = 0..101;

    // Parse each player cell data
    for data in cell_data {
        rows += 1;
        // Remove commas and store complete numbers in vector
        let split_string: Vec<&str> = data.split(',').collect();
        // Check that cells in save can fit on game board width
        if width as usize - 1 != split_string.len() {
            return false;
        }
        for player in split_string {
            // Attempt to convert the player cell entry to numeric
            let Ok(player_as_numeric) = player.parse::<u8>() else {
                return false;
            };
            // Ensure numbers are within valid range and the number of players is less than or equal to the player count
            if !valid_cell_entries.contains(&player_as_numeric) || player_as_numeric > player_count
            {
                return false;
            }
        }
    }

    // Check that cells in save can fit on game board height
    if height - 1 != rows {
        return false;
    }

    true
}
