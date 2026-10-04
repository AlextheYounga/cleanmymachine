use std::error::Error;

use cleanmymachine::app;

fn main() -> Result<(), Box<dyn Error>> {
    app::run()
}
