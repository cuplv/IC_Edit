extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

mod functional;

use std::env::current_exe;
use glutin_window::GlutinWindow;
use graphics::Transformed;
use opengl_graphics::{GlGraphics, OpenGL};
use opengl_graphics::glyph_cache::GlyphCache;
use piston::event_loop::{Events, EventLoop};
use piston::input::{Button, Event, Input, Key};
use piston::window::WindowSettings;

use functional::List;

const OPEN_GL: OpenGL = OpenGL::V3_2;

type Cursor = String;

#[derive(Debug)]
enum Symbol {
	Cur(Cursor),
	Data(String),
}

//struct Zip;

fn build_content(keys: &List<Symbol>) -> String {
  fn build(ks: &List<Symbol>, c: String) -> String {
    if let Some(k) = ks.head() {
      match k {
        &Symbol::Cur(_) => build(&ks, c),
        //TODO: find a more elegant way to copy s
        &Symbol::Data(ref s) => build(&ks.tail(), s.to_string() + &c)
      }
    } else {c}
  }
  build(keys, "".to_string())
}

// Returns a result containing a GlutinWindow or an error if the window
// settings are not supported
fn try_create_window() -> Result<GlutinWindow, String> {
  WindowSettings::new("ICEdit", [800, 600])
    .exit_on_esc(true)
    .opengl(OPEN_GL)
    .build()
}

fn render(c: graphics::context::Context, g: &mut GlGraphics, f: &mut GlyphCache, t: &String) {
  graphics::clear([0.0, 0.0, 0.0, 1.0], g);

  //println!("{:?}", &t);

  let mut text = graphics::Text::new(22);
  text.color = [1.0, 0.0, 0.0, 1.0];
  text.draw(
    &t,
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

  let mut needs_update = true;
  let mut command_key_down = false;
  let mut inputs = List::new();
  let mut content_text = "".to_string();

  for e in window.events().max_fps(60) {
    match e {
      //gives typed char or empty
      Event::Input(Input::Text(t)) => {
        if t == "" || command_key_down {continue}
        inputs = inputs.append(Symbol::Data(t));
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
            command_key_down = false;
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
            command_key_down = true;
          }
          Key::Left |
          Key::Right |
          Key::Up |
          Key::Down |
          Key::Backspace  => {
            let m = if command_key_down {"C: "} else {""};
            println!("{}{:?}", m, key);
            needs_update = true;
          }
          Key::Return => {
            if command_key_down {println!("C: Return");}
            else {
              inputs = inputs.append(Symbol::Data("\n".to_string()));
              needs_update = true;
            }
          }
          _ => {
            if command_key_down {
              println!("C: {:?}", key);
              needs_update = true;
            }
          }
        }
      }
      Event::Render(args) => {
        if needs_update {
          content_text = build_content(&inputs);
          needs_update = false
        }
        gl.draw(args.viewport(), |c, g| render(c, g, &mut font, &content_text));
        
      }
      _ => {}

    }
  }

}