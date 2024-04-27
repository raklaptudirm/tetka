mod gui;
use gui::*;

fn main() {
    let mut gui = GuiBuilder::new().build();

    while !gui.should_quit() {
        let mut drawer = gui.new_drawer();
        drawer.draw();
    }
}
