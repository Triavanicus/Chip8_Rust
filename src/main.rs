mod app;
mod chip8;

use app::App;

// Welcome ladies, gentlemen, and others
fn main() -> Result<(), std::io::Error> {
    // Here we create a new instance of this application
    let mut app = App::new();
    // And run it
    app.run()
}
