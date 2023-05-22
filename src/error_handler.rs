use std::process;

pub fn handle_error(error: u8) {
    match error {
        1 => eprintln!("Usage: boxes height width playercount [filename]"),
        2 => eprintln!("Invalid grid dimensions"),
        3 => eprintln!("Invalid player count"),
        4 => eprintln!("Invalid grid file"),
        5 => eprintln!("Error reading grid contents"),
        6 => eprintln!("End of user input"),
        9 => eprintln!("System call failure"),
        _ => eprintln!("Unhandled error!"),
    }

    process::exit(error.into());
}
