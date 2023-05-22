use std::env;

use boxes::configuration;
use boxes::error_handler::handle_error;
use boxes::game::run;
fn main() {
    let config = match configuration::Config::build(env::args()) {
        Ok(config) => config,
        Err(e) => return handle_error(e),
    };

    match run(config) {
        Ok(_) => (),
        Err(e) => handle_error(e),
    }
}
