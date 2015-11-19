extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use std::env::current_exe;
use glutin_window::GlutinWindow;
use graphics::Transformed;
use opengl_graphics::{GlGraphics, OpenGL};
use opengl_graphics::glyph_cache::GlyphCache;
use piston::event_loop::{Events, EventLoop};
use piston::input::{Button, Event, Input, RenderEvent, Key};
use piston::window::WindowSettings;

const OPEN_GL: OpenGL = OpenGL::V3_2;

type Cursor = String;

enum Symbol {
	Cur(Cursor),
	Data(String),
}

// Returns a result containing a GlutinWindow or an error if the window
// settings are not supported
fn try_create_window() -> Result<GlutinWindow, String> {
  WindowSettings::new("ICEdit", [800, 600])
    .exit_on_esc(true)
    .opengl(OPEN_GL)
    .build()
}

fn render(c: graphics::context::Context, g: &mut GlGraphics, f: &mut GlyphCache, count: i32) {
  graphics::clear([0.0, 0.0, 0.0, 1.0], g);

  let mut text = graphics::Text::new(22);
  text.color = [1.0, 0.0, 0.0, 1.0];
  text.draw(
    &format!("Update count: {}", count),
    f,
    &c.draw_state,
    c.trans(10.0, 20.0).transform,
    g);
}

fn main() {
    
  let window = try_create_window().unwrap();
  let mut gl = GlGraphics::new(OPEN_GL);
  let exe_directory = current_exe().unwrap().parent().unwrap().to_owned();
  let mut font = GlyphCache::new(&exe_directory.join("../../FiraMono-Bold.ttf")).unwrap();

  let mut count = 0;
  let mut needs_update = true;
  let mut command_key = false;

  for e in window.events().max_fps(60) {
    match e {
      //gives typed char or empty
      Event::Input(Input::Text(t)) => {
        if t == "" || command_key {continue}
        println!("{:?}", t);
        needs_update = true;
      }
      Event::Input(Input::Release(Button::Keyboard(key))) => {
        match key {
          //mac's command key registers as unknown on my machine
          Key::Unknown |
          Key::LCtrl |
          Key::LAlt |
          Key::RCtrl |
          Key::RAlt => {
            command_key = false;
          }
          _ => {}
        }
      }
      Event::Input(Input::Press(Button::Keyboard(key))) => {
        match key {
          //mac's command key registers as unknown on my machine
          Key::Unknown |
          Key::LCtrl |
          Key::LAlt |
          Key::RCtrl |
          Key::RAlt => {
            command_key = true;
          }
          Key::Left |
          Key::Right |
          Key::Up |
          Key::Down |
          Key::Backspace |
          Key::Return => {
            let m = if command_key {"C: "} else {""};
            println!("{}{:?}", m, key);
            needs_update = true;
          }
          _ => {
            if command_key {
              println!("C: {:?}", key);
              needs_update = true;
            }
          }
        }
      }
      Event::Render(args) => {
        //if !needs_update {continue}
        //println!("render");
        if needs_update {count += 1}
        gl.draw(args.viewport(), |c, g| render(c, g, &mut font, count));
        needs_update = false;
      }
      _ => {}

    }
  }

}