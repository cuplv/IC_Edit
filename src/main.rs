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
enum Dir {
  L,
  R,
}

#[derive(Debug)]
enum Symbol {
	Cur(Cursor),
	Data(String),
}

#[derive(Debug)]
enum Action {
  Cmd(Command),
  Undo,
  Redo,
}

#[derive(Debug)]
enum Command {
  Ins(String, Dir),
  Rem(Dir),
  Move(Dir),
  Repl(String, Dir),
  Mk(Cursor),
  Switch(Cursor),
  Jmp(Cursor),
  Join(Cursor),
}

type Zip<T> = (List<T>,List<T>);
type CZip<T> = (List<T>,Cursor,List<T>);

fn build_content(keys: &List<Action>) -> List<String> {
  println!("Produce text"); "".to_string();
  // fn build(ks: &List<Symbol>, c: String) -> String {
  //   if let Some(k) = ks.head() {
  //     match k {
  //       &Symbol::Cur(_) => build(&ks, c),
  //       //TODO: find a more elegant way to copy s
  //       &Symbol::Data(ref s) => build(&ks.tail(), s.to_string() + &c)
  //     }
  //   } else {c}
  // }
  // build(keys, "".to_string())
  List::new()
    .append("nothing here".to_string())
    .append("or here".to_string())
    .rev()
}

fn render(c: graphics::context::Context, g: &mut GlGraphics, f: &mut GlyphCache, t: &List<String>) {
  graphics::clear([0.0, 0.0, 0.0, 1.0], g);

  //println!("{:?}", &t);

  let mut text = graphics::Text::new(22);
  text.color = [1.0, 1.0, 1.0, 1.0];
  let mut loc = 20.0;

  for st in t.iter() {
    text.draw(
      st,
      f,
      &c.draw_state,
      c.trans(10.0, loc).transform,
      g); 
    loc += 20.0;
    if loc > 800.0 {break}
  }
}

// Returns a result containing a GlutinWindow or an error if the window
// settings are not supported
fn try_create_window() -> Result<GlutinWindow, String> {
  WindowSettings::new("ICEdit", [800, 800])
    .exit_on_esc(true)
    .opengl(OPEN_GL)
    .build()
}

fn main() {
    
  let window = try_create_window().unwrap();
  let mut gl = GlGraphics::new(OPEN_GL);
  let exe_directory = current_exe().unwrap().parent().unwrap().to_owned();
  let mut font = GlyphCache::new(&exe_directory.join("../../FiraMono-Bold.ttf")).unwrap();

  let mut needs_update = true;
  let mut command_key_down = false;
  let mut inputs = List::new();
  let mut content_text = List::new().append("".to_string());

  for e in window.events().max_fps(60) {
    match e {
      //gives typed char or empty
      Event::Input(Input::Text(t)) => {
        if t == "" || command_key_down {continue}
        inputs = inputs.append(Action::Cmd(Command::Ins(t,Dir::L)));
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
          Key::Down => {
            let m = if command_key_down {"C: "} else {""};
            println!("{}{:?}", m, key);
          }
          Key::Backspace  => {
            if command_key_down {println!("C: Backspace");}
            else{
              inputs = inputs.append(
                Action::Cmd(Command::Rem(Dir::L))
              );
              needs_update = true;
            }
          }
          Key::Return => {
            if command_key_down {println!("C: Return");}
            else {
              inputs = inputs.append(
                Action::Cmd(Command::Ins("\n".to_string(), Dir::L))
              );
              needs_update = true;
            }
          }
          Key::Z => {
            if command_key_down {
              inputs = inputs.append(Action::Undo);
              needs_update = true;
            } else {continue}
          }
          Key::Y => {
            if command_key_down {
              inputs = inputs.append(Action::Redo);
              needs_update = true;
            } else {continue}
          }
          _ => {
            if command_key_down {
              println!("C: {:?}", key);
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