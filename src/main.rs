use std::env;

use boxes::configuration;
use boxes::error_handler::handle_error;
use boxes::game::run;
fn main() {
    let config = match configuration::Config::build(env::args()) {
        Ok(config) => config,
        Err(e) => return handle_error(e),
    };

    let winners: String = match run(config) {
        Ok(winners) => winners,
        Err(e) => return handle_error(e),
    };

    println!("Winner(s): {winners}");
}
