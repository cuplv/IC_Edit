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

impl Dir {
  fn opp(&self) -> Dir {
    match *self {
      Dir::L => {Dir::R}
      Dir::R => {Dir::L}
    }
  }
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
  Ins(String, Dir),   //Insert <String>, moving cursor <Dir>
  Rem(Dir),           //Remove character located <Dir>
  Move(Dir),          //Move cursor <Dir>
  Ovr(String, Dir),   //Overwrite with <String>, moving cursor <Dir>
  Mk(Cursor),
  Switch(Cursor),
  Jmp(Cursor),
  Join(Cursor),
}

#[derive(Debug, Clone)]
enum CCs {
  Mk, Switch, Jmp, Join
}

enum Inputstatus {
  Insert(Dir, bool),    //Mode, direction, showcursors
  Overwrite(Dir, bool),
  EnterCursor(
    Box<Inputstatus>,   // prior input status
    CCs,                // current command in process
    List<Action>,       // actions generating new cursor
    String,             // new cursor in progress
  )
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
          None => {(List::new(),buffer)}
          Some(d) => {
            (content.tail(),
              buffer.append(d.clone()))
          }
        }
      },
      Action::Redo => {
        match buffer.head() {
          None => {(content,List::new())}
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

//move the zipper through cursors in the given direction 
fn passthrough(direction: Dir, before: List<Symbol>, after: List<Symbol>)
  -> (List<Symbol>, List<Symbol>) {
  let mut head;
  let mut first;
  let mut second;

  match direction {
    Dir::L => {
      first = before;
      second = after;
    }
    Dir::R => {
      first = after;
      second = before;
    }
  }

  loop {
    head = first.head().map(|h| h.clone());
    match head {
      Some(Symbol::Cur(c)) => {
        first = first.tail();
        second = second.append(Symbol::Cur(c.clone()));
      }
      _ => {break}
    }
  }

  match direction {
    Dir::L => {(first, second)}
    Dir::R => {(second, first)}
  }
}

fn join_cursor(cur: Cursor, l: &List<Symbol>, r: &List<Symbol>)
  -> Option<(List<Symbol>, List<Symbol>)> {
  let mut first = l.clone();
  let mut second = r.clone();

  //search left
  loop {
    match first.head().map(|h| h.clone()) {
      None => {break}
      Some(Symbol::Data(d)) => {
        first = first.tail();
        second = second.append(Symbol::Data(d));
      }
      Some(Symbol::Cur(c)) => {
        if cur == c {
          return Some((first.tail(),second))
        }else{
          first = first.tail();
          second = second.append(Symbol::Cur(c));
        }
      }
    }
  }
  //search right
  first = l.clone();
  second = r.clone();
  loop {
    match second.head().map(|h| h.clone()) {
      None => {return None}
      Some(Symbol::Data(d)) => {
        second = second.tail();
        first = first.append(Symbol::Data(d));
      }
      Some(Symbol::Cur(c)) => {
        if cur == c {
          return Some((first,second.tail()))
        }else{
          second = second.tail();
          first = first.append(Symbol::Cur(c));
        }
      }
    }
  }

}


// command list to content zipper
fn cl_to_cz(commands: &List<Command>) -> CZip<Symbol> {
  let mut before: List<Symbol> = List::new();
  let mut ccursor: Cursor = "0".to_string();
  let mut after: List<Symbol> = List::new();

  for command in commands.iter() {
    let (before2, ccursor2, after2) =
    match *command {
      Command::Ins(ref d, Dir::R) => {
        let (b, a) = passthrough(Dir::R, before, after);
        (b.append(Symbol::Data(d.clone())), ccursor, a)
      }
      Command::Ins(ref d, Dir::L) => {
        let (b, a) = passthrough(Dir::L, before, after);
        (b, ccursor, a.append(Symbol::Data(d.clone())))
      }
      Command::Rem(Dir::L) => {
        let (b, a) = passthrough(Dir::L, before, after);
        (b.tail(), ccursor, a)
      }
      Command::Rem(Dir::R) => {
        let (b, a) = passthrough(Dir::R, before, after);
        (b, ccursor, a.tail())
      }
      Command::Move(Dir::L) => {
        let (b, a) = passthrough(Dir::L, before, after);
        match b.head() {
          None => {(List::new(), ccursor, a)}
          Some(d) => {(b.tail(), ccursor, a.append(d.clone()))}
        }
      }
      Command::Move(Dir::R) => {
        let (b, a) = passthrough(Dir::R, before, after);
        match a.head() {
          None => {(b, ccursor, List::new())}
          Some(d) => {(b.append(d.clone()), ccursor, a.tail())}
        }
      }
      Command::Ovr(ref d, Dir::L) => {
        let (b, a) = passthrough(Dir::L, before, after);
        (b.tail(), ccursor, a.append(Symbol::Data(d.clone())))
      }
      Command::Ovr(ref d, Dir::R) => {
        let (b, a) = passthrough(Dir::R, before, after);
        (b.append(Symbol::Data(d.clone())), ccursor, a.tail())
      }
      Command::Mk(ref c) => {
        (before.append(Symbol::Cur(c.clone())), ccursor, after)
      }
      // Switch(Cursor),
      // Jmp(Cursor),
      Command::Join(ref c) => {
        match join_cursor(c.clone(), &before, &after) {
          Some((b,a)) => {(b, c.clone(), a)}
          None => {(before, ccursor, after)}
        }
      }
      _ => {println!("Unsupported opperation");(before, ccursor, after)}
    };

    before = before2;
    after = after2;
    ccursor = ccursor2;
  }

  (before, ccursor, after)
}

fn makelines(before: &List<Symbol>, after: &List<Symbol>, addbar: bool, showcursors: bool) -> List<String> {
  let mut out: List<String> = List::new();
  let mut partial: String = "".to_string();

  for s in after.iter() {
    match *s {
      Symbol::Cur(ref c) => {
        if showcursors {partial = partial + "<" + &c + ">"}
      }
      Symbol::Data(ref d) => {
        if d == "\n" {
          out = out.append(partial);
          partial = "".to_string();
        } else {partial = partial + &d}
      }
    }
  }
  out = out.append(partial).rev();

  //concat the two sides with cursor
  let cur = if addbar {"|"} else {""};
  match out.head(){
    None => {partial = cur.to_string();}
    Some(t) => {partial = cur.to_string() + t;}
  }
  out = out.tail();

  for s in before.iter() {
    match *s {
      Symbol::Cur(ref c) => {
        if showcursors {partial = "<".to_string() + &c + ">" + &partial}
      }
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

fn build_content(keys: &List<Action>, addcursor: bool, showcursors: bool) -> List<String> {
  let (commands, _) = al_to_ub(&keys.rev());
  let (before, _, after) = cl_to_cz(&commands.rev());
  makelines(&before, &after, addcursor, showcursors)
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

fn render_cursor(c: graphics::context::Context, g: &mut GlGraphics, f: &mut GlyphCache, cc:CCs, t: &String) {
  graphics::clear([0.0, 0.0, 0.0, 1.0], g);

  //println!("{:?}", &t);
  let size = 32.0;
  let mut text = graphics::Text::new(32);
  text.color = [1.0, 1.0, 1.0, 1.0];
  let (px,py) = (200.0,250.0);
  let prompt = match cc {
    CCs::Mk => {"Create cursor: "}
    CCs::Switch => {"Switch to cursor: "}
    CCs::Jmp => {"Jump to cursor: "}
    CCs::Join => {"Join with cursor: "}
  }.to_string();

  text.draw(
    &prompt,
    f,
    &c.draw_state,
    c.trans(px, py).transform,
    g); 
  text.draw(
    t,
    f,
    &c.draw_state,
    c.trans(px + size, py + size*1.5).transform,
    g); 
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
  let mut status = Inputstatus::Insert(Dir::R, false);
  let mut inputs = List::new();
  let mut content_text = List::new().append("".to_string());

  for e in window.events().max_fps(60) {
    match e {
      //gives typed char or empty
      Event::Input(Input::Text(t)) => {
        if t == "" || command_key_down {continue}
        status = match status {
          Inputstatus::Insert(d, s) => {
            inputs = inputs.append(Action::Cmd(Command::Ins(t,d.clone())));
            Inputstatus::Insert(d, s)
          }
          Inputstatus::Overwrite(d, s) => {
            inputs = inputs.append(Action::Cmd(Command::Ovr(t,d.clone())));
            Inputstatus::Overwrite(d, s)
          }
          Inputstatus::EnterCursor(p,c,a,_) => {
            let a2 = a.append(Action::Cmd(Command::Ins(t,Dir::R)));
            let content = build_content(&a2, true, false).head().unwrap_or(&"".to_string()).clone();
            Inputstatus::EnterCursor(
              p,c,a2,
              content
            )
          }
        };
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
          //command keys
          //mac's command key registers as unknown on my machine
          Key::Unknown |
          Key::LCtrl |
          Key::LAlt |
          Key::RCtrl |
          Key::RAlt => {
            command_key_down = true;
          }

          Key::Up => {
            if command_key_down {
              println!("Mode: Overwrite");
              status = match status {
                Inputstatus::Insert(d, s) | Inputstatus::Overwrite(d, s) => {
                  Inputstatus::Overwrite(d, s)
                }
                Inputstatus::EnterCursor(p,c,a,ct) => {
                  Inputstatus::EnterCursor(p,c,a,ct)                  
                }
              };
            } else {
              println!("{:?}", key)
            }
          }
          Key::Down => {
            if command_key_down {
              println!("Mode: Insert");
              status = match status {
                Inputstatus::Insert(d, s) | Inputstatus::Overwrite(d, s) => {
                  Inputstatus::Insert(d, s)
                }
                Inputstatus::EnterCursor(p,c,a,ct) => {
                  Inputstatus::EnterCursor(p,c,a,ct)                  
                }
              };
            } else {
              println!("{:?}", key)
            }
          }
          Key::Left => {
            if command_key_down {
              println!("Mode: Left");
              status = match status {
                Inputstatus::Insert(_, s) => {
                  Inputstatus::Insert(Dir::L, s)
                }
                Inputstatus::Overwrite(_, s) => {
                  Inputstatus::Overwrite(Dir::L, s)
                }
                Inputstatus::EnterCursor(p,c,a,ct) => {
                  Inputstatus::EnterCursor(p,c,a,ct)                  
                }
              };
            }
            else{
              status = match status {
                Inputstatus::Insert(d, s) => {
                  inputs = inputs.append(
                    Action::Cmd(Command::Move(Dir::L))
                  );
                  Inputstatus::Insert(d, s)
                }
                Inputstatus::Overwrite(d, s) => {
                  inputs = inputs.append(
                    Action::Cmd(Command::Move(Dir::L))
                  );
                  Inputstatus::Overwrite(d, s)
                }
                Inputstatus::EnterCursor(p,c,a,_) => {
                  let a2 = a.append(Action::Cmd(Command::Move(Dir::L)));
                  let content = build_content(&a2,true,false).head().unwrap_or(&"".to_string()).clone();
                  Inputstatus::EnterCursor(
                    p,c,a2,
                    content
                  )                  
                }
              };
              needs_update = true;
            }
          }
          Key::Right => {
            if command_key_down {
              println!("Mode: Right");
              status = match status {
                Inputstatus::Insert(_, s) => {
                  Inputstatus::Insert(Dir::R, s)
                }
                Inputstatus::Overwrite(_, s) => {
                  Inputstatus::Overwrite(Dir::R, s)
                }
                Inputstatus::EnterCursor(p,c,a,ct) => {
                  Inputstatus::EnterCursor(p,c,a,ct)                  
                }
              };
            }
            else{
              status = match status {
                Inputstatus::Insert(d, s) => {
                  inputs = inputs.append(
                    Action::Cmd(Command::Move(Dir::R))
                  );
                  Inputstatus::Insert(d, s)
                }
                Inputstatus::Overwrite(d, s) => {
                  inputs = inputs.append(
                    Action::Cmd(Command::Move(Dir::R))
                  );
                  Inputstatus::Overwrite(d, s)
                }
                Inputstatus::EnterCursor(p,c,a,_) => {
                  let a2 = a.append(Action::Cmd(Command::Move(Dir::R)));
                  let content = build_content(&a2,true, false).head().unwrap_or(&"".to_string()).clone();
                  Inputstatus::EnterCursor(
                    p,c,a2,
                    content
                  )                  
                }
              };
              needs_update = true;
            }
          }
          Key::D => {
            if command_key_down {
              status = match status {
                Inputstatus::Insert(d, s) => {
                  inputs = inputs.append(
                    Action::Cmd(Command::Rem(d.clone()))
                  );
                  Inputstatus::Insert(d, s)
                }
                Inputstatus::Overwrite(d, s) => {
                  inputs = inputs.append(
                    Action::Cmd(Command::Rem(d.clone()))
                  );
                  Inputstatus::Overwrite(d, s)
                }
                Inputstatus::EnterCursor(p,c,a,_) => {
                  let a2 = a.append(Action::Cmd(Command::Rem(Dir::R)));
                  let content = build_content(&a2,true, false).head().unwrap_or(&"".to_string()).clone();
                  Inputstatus::EnterCursor(
                    p,c,a2,
                    content
                  )                  
                }
              };
              needs_update = true;
            } else {continue}
          }
          Key::Backspace  => {
            if command_key_down {println!("C: Backspace");}
            else{
              status = match status {
                Inputstatus::Insert(d, s) => {
                  inputs = inputs.append(
                    Action::Cmd(Command::Rem(d.opp()))
                  );
                  Inputstatus::Insert(d, s) 
                }
                Inputstatus::Overwrite(d, s) => {
                  inputs = inputs.append(
                    Action::Cmd(Command::Rem(d.opp()))
                  );
                  Inputstatus::Overwrite(d, s) 
                }
                Inputstatus::EnterCursor(p,c,a,_) => {
                  let a2 = a.append(Action::Cmd(Command::Rem(Dir::L)));
                  let content = build_content(&a2, true, false).head().unwrap_or(&"".to_string()).clone();
                  Inputstatus::EnterCursor(
                    p,c,a2,
                    content
                  )                  
                }
              };
              needs_update = true;
            }
          }
          Key::Return => {
            if command_key_down {println!("C: Return");}
            else {
              status = match status {
                Inputstatus::Insert(d, s) => {
                  inputs = inputs.append(
                    Action::Cmd(Command::Ins("\n".to_string(), d.clone()))
                  );
                  Inputstatus::Insert(d, s)
                }
                Inputstatus::Overwrite(d, s) => {
                  inputs = inputs.append(
                    Action::Cmd(Command::Ins("\n".to_string(), d.clone()))
                  );
                  Inputstatus::Overwrite(d, s)
                }
                Inputstatus::EnterCursor(p,c,a,_) => {
                  let content = build_content(&a, false, false).head().unwrap_or(&"".to_string()).clone();
                  let newcommand = match c {
                      CCs::Mk => {Command::Mk(content)}
                      CCs::Switch => {Command::Switch(content)}
                      CCs::Jmp => {Command::Jmp(content)}
                      CCs::Join => {Command::Join(content)}
                  };
                  inputs = inputs.append(Action::Cmd(newcommand));
                  *p
                }
              };
              needs_update = true;
            }
          }
          Key::Z => {
            if command_key_down {
              let newstatus = match status {
                Inputstatus::Insert(_, _) | Inputstatus::Overwrite(_, _) => {
                  inputs = inputs.append(Action::Undo);
                  status
                }
                Inputstatus::EnterCursor(p,cc,a,_) => {
                  let a2 = a.append(Action::Undo);
                  let content = build_content(&a2, true, false).head().unwrap_or(&"".to_string()).clone();
                  Inputstatus::EnterCursor(
                    p,cc,a2,
                    content
                  )
                }
              };
              status = newstatus;
              needs_update = true;
            } else {continue}
          }
          Key::Y => {
            if command_key_down {
              let newstatus = match status {
                Inputstatus::Insert(_, _) | Inputstatus::Overwrite(_, _) => {
                  inputs = inputs.append(Action::Redo);
                  status
                }
                Inputstatus::EnterCursor(p,cc,a,_) => {
                  let a2 = a.append(Action::Redo);
                  let content = build_content(&a2, true, false).head().unwrap_or(&"".to_string()).clone();
                  Inputstatus::EnterCursor(
                    p,cc,a2,
                    content
                  )
                }
              };
              status = newstatus;
              needs_update = true;
            } else {continue}
          }
          /*Mk*/ Key::M => {
            if command_key_down{
                let newstatus = match status {
                  Inputstatus::Insert(_, _) | Inputstatus::Overwrite(_, _) => {
                    Inputstatus::EnterCursor(Box::new(status), CCs::Mk, List::new(), "".to_string())
                  }
                  Inputstatus::EnterCursor(_,_,_,_) => {
                    status
                  }
                };
                status = newstatus;
                needs_update = true;
            } else {continue}
          }
          /*Switch*/ Key::H |
          /*Jmp*/ Key::J =>
          {
            if command_key_down {
              println!("C: {:?}", key);
            } else {continue};
          }
          /*Join*/ Key::N => {
            if command_key_down {
              let newstatus = match status {
                Inputstatus::Insert(_, _) | Inputstatus::Overwrite(_, _) => {
                  Inputstatus::EnterCursor(Box::new(status), CCs::Join, List::new(), "".to_string())
                }
                Inputstatus::EnterCursor(_,_,_,_) => {
                  status
                }
              };
              status = newstatus;
              needs_update = true;
           }else{continue}
          } 
          Key::S => {
            if command_key_down{
                status = match status {
                  Inputstatus::Insert(d, s) => {
                    Inputstatus::Insert(d, !s)
                  }
                  Inputstatus::Overwrite(d, s) => {
                    Inputstatus::Overwrite(d, !s)
                  }
                  Inputstatus::EnterCursor(p,c,a,ct) => {
                    Inputstatus::EnterCursor(p,c,a,ct)
                  }
                };
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
        match status {
          Inputstatus::Insert(_, s) | Inputstatus::Overwrite(_, s) => {
            if needs_update {
              content_text = build_content(&inputs, true, s);
              needs_update = false
            }
            gl.draw(args.viewport(), |c, g| render(c, g, &mut font, &content_text));
          }
          Inputstatus::EnterCursor(_, ref cc, _, ref ct) => {           
            gl.draw(args.viewport(), |c, g| render_cursor(c, g, &mut font, cc.clone(), ct));
          }
        }
        
      }
      _ => {}

    }
  }

}