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

#[derive(Debug, Clone)]
enum Dir {
  L,
  R,
}

#[derive(Debug, Clone)]
enum Symbol {
	Cur(Cursor),
	Data(String),
}

#[derive(Debug, Clone)]
enum Action {
  Cmd(Command),
  Undo,
  Redo,
}

#[derive(Debug, Clone)]
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

//Action list to undo buffer
fn al_to_ub(acts: &List<Action>) -> Zip<Command> {
  let mut content: List<Command> = List::new();
  let mut buffer: List<Command> = List::new();

  for act in acts.iter() {
    let (content2, buffer2) =
    match *act {
      Action::Undo => {
        match content.head() {
          None => {println!("Can't Undo");(List::new(),buffer)}
          Some(d) => {
            (content.tail(),
              buffer.append(d.clone()))
          }
        }
      },
      Action::Redo => {
        match buffer.head() {
          None => {println!("Can't Redo");(content,List::new())}
          Some(d) => {
            (content.append(d.clone()),
              buffer.tail())
          }
        }
      },
      Action::Cmd(ref c) => {
        (content.append(c.clone()),
          List::new())
      }
    };
    content = content2;
    buffer = buffer2;
  }
  (content,buffer)
}

// command list to content zipper
fn cl_to_cz(commands: &List<Command>) -> CZip<Symbol> {
  let mut before = List::new();
  let mut after = List::new();

  for command in commands.iter() {
    let (before2, after2) =
    match *command {
      Command::Ins(ref d, Dir::L) => {
        (before.append(Symbol::Data(d.clone())),
          after)
      }
      Command::Rem(Dir::L) => {
        (before.tail(),
          after)
      }
      Command::Move(Dir::L) => {
        match before.head() {
          None => {println!("At start");(List::new(),after)}
          Some(d) => {
            (before.tail(),
              after.append(d.clone()))
          }
        }
      }
      Command::Move(Dir::R) => {
        match after.head() {
          None => {println!("At end");(before,List::new())}
          Some(d) => {
            (before.append(d.clone()),
              after.tail())
          }
        }
      }
      Command::Repl(ref d, Dir::L) => {
        (before.tail().append(Symbol::Data(d.clone())),
          after)
      }
      Command::Repl(ref d, Dir::R) => {
        (before,
          after.tail().append(Symbol::Data(d.clone())))
      }
      _ => {println!("Unsupported opperation");(before,after)}
      // Mk(Cursor),
      // Switch(Cursor),
      // Jmp(Cursor),
      // Join(Cursor),
    };

    before = before2;
    after = after2;
  }

  (before, "0".to_string(), after)
}

fn makelines(before: &List<Symbol>, after: &List<Symbol>) -> List<String> {
  let mut out: List<String> = List::new();
  let mut partial: String = "".to_string();

  for s in after.iter() {
    match *s {
      Symbol::Cur(_) => {}
      Symbol::Data(ref d) => {
        if d == "\n" {
          out = out.append(partial);
          partial = "".to_string();
        } else {partial = partial + &d}}
    }
  }
  out = out.append(partial).rev();

  //concat the two sides with cursor
  match out.head(){
    None => {partial = "|".to_string();}
    Some(t) => {partial = "|".to_string() + t;}
  };
  out = out.tail();

  for s in before.iter() {
    match *s {
      Symbol::Cur(_) => {}
      Symbol::Data(ref d) => {
        if d == "\n" {
          out = out.append(partial);
          partial = "".to_string();
        } else {partial = d.clone() + &partial}
      }
    }
  }
  out = out.append(partial);

  out
}

fn build_content(keys: &List<Action>) -> List<String> {
  let (commands, _) = al_to_ub(&keys.rev());
  let (before, _, after) = cl_to_cz(&commands.rev());
  makelines(&before, &after)
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
          Key::Up |
          Key::Down => {
            let m = if command_key_down {"C: "} else {""};
            println!("{}{:?}", m, key);
          }
          Key::Left => {
            if command_key_down {println!("C: Left");}
            else{
              inputs = inputs.append(
                Action::Cmd(Command::Move(Dir::L))
              );
              needs_update = true;
            }
          }
          Key::Right => {
            if command_key_down {println!("C: Right");}
            else{
              inputs = inputs.append(
                Action::Cmd(Command::Move(Dir::R))
              );
              needs_update = true;
            }
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