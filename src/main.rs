mod app;
mod chip8;

use app::App;

fn main() -> Result<(), std::io::Error> {
    let mut app = App::new();
    app.run()
}
