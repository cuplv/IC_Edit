// Default parameters
// ------------------

// icedit -x <Num> / --width <Num>
// Width in pixels
const DEFAULT_WIDTH: u32 = 800;

// icedit -y <Num> / --height <Num>
// Height in pixels
const DEFAULT_HEIGHT: u32 = 800;

// icedit test -s <Num> / --rnd_start <Num>
// number of random starting commands
const DEFAULT_RND_START: u32 = 0000;

// icedit test -a <Num> / --rnd_adds <Num>
// nummer of random commands after start
const DEFAULT_RND_ADDITIONS: u32 = 0;


extern crate time;
extern crate rand;
#[macro_use]
extern crate clap;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate adapton;

mod functional;
mod spec;

use std::env::current_exe;
use time::Duration;
use glutin_window::GlutinWindow;
use graphics::Transformed;
use opengl_graphics::{GlGraphics, OpenGL};
use opengl_graphics::glyph_cache::GlyphCache;
use piston::event_loop::{Events, EventLoop};
use piston::input::{Button, Event, Input, Key};
use piston::window::WindowSettings;
use functional::List;

use spec::*;

const OPEN_GL: OpenGL = OpenGL::V3_2;

fn makelines(before: &List<Symbol>, after: &List<Symbol>, addbar: bool, showcursors: bool) -> List<String> {
  let mut out: List<String> = List::new();
  let mut partial: String = "".to_string();
  let mut count = 40; //HACK: draw off the screen sometimes to make sure the screen is full

  for s in after.iter() {
    match *s {
      Symbol::Cur(ref c) => {
        if showcursors {partial = partial + "<" + &c + ">"}
      }
      Symbol::Data(ref d) => {
        if d == "\n" {
          out = out.append(partial);
          partial = "".to_string();
          count = count - 1;
          if count <= 0 {break}
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

  count = 20; //cursor no lower than half screen
  for s in before.iter() {
    match *s {
      Symbol::Cur(ref c) => {
        if showcursors {partial = "<".to_string() + &c + ">" + &partial}
      }
      Symbol::Data(ref d) => {
        if d == "\n" {
          out = out.append(partial);
          partial = "".to_string();
          count = count - 1;
          if count <= 0 {break}
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

fn rnd_inputs(num: u32) -> List<Action> {
  use rand::{Rng, ThreadRng, thread_rng};
  let mut rng = thread_rng();
  let mut cursor_count = 1;
  let mut acts = List::new();

  fn rnd_cursor(rng: &mut ThreadRng) -> Cursor {
    let rn: u8 = rng.gen_range(48,58); //numbers
    String::from_utf8(vec![rn]).unwrap()
  }

  fn rnd_char(rng: &mut ThreadRng) -> String {
    let ascii: u8 = match rng.gen_range(0, 20) {
      0 ... 4 => {32} //space
      5 ... 6 => {rng.gen_range(48,48+10)} //numbers
      7 ... 16 => {rng.gen_range(97,97+26)} //lower case
      17 ... 18 => {rng.gen_range(65,65+26)} //upper case
      _ => {13} //return
    };
    if ascii == 13 {"\n".to_string()} else {
      String::from_utf8(vec![ascii]).unwrap()  
    }
  }

  fn rnd_dir(rng: &mut ThreadRng) -> Dir {
    if rng.gen() {Dir::R} else {Dir::L}
  }

  let mut rnd_action = |rng: &mut ThreadRng|{//(&rng: Rng) -> Action {
    match rng.gen_range(0, 100) {
      0 ... 17 => {Action::Cmd(Command::Ovr(rnd_char(rng), rnd_dir(rng)))}
      18 ... 62 => {Action::Cmd(Command::Ins(rnd_char(rng), rnd_dir(rng)))}
      63 ... 80 => {Action::Cmd(Command::Rem(rnd_dir(rng)))}
      61 ... 98 => {Action::Cmd(Command::Move(rnd_dir(rng)))}
      _ => match rng.gen_range(0, 3) {
        0 => {
          cursor_count = cursor_count + 1;
          Action::Cmd(Command::Mk((cursor_count - 1).to_string()))
        }
        1 => {Action::Cmd(Command::Switch(rnd_cursor(rng)))}
        2 => {Action::Cmd(Command::Jmp(rnd_cursor(rng)))}
        _ => {Action::Undo}
      }
    }
  };

  for _ in 0..num {
    acts = acts.append(rnd_action(&mut rng));
  }
  acts
}

fn render(c: graphics::context::Context, g: &mut GlGraphics, f: &mut GlyphCache, t: &List<String>, time: Duration) {
  graphics::clear([0.0, 0.0, 0.0, 1.0], g);

  //println!("{:?}", &t);

  let size = 22.0;
  let mut text = graphics::Text::new(22);
  text.color = [1.0, 1.0, 1.0, 1.0];
  let mut loc = size;

  for st in t.iter() {
    text.draw(
      st,
      f,
      &c.draw_state,
      c.trans(10.0, loc).transform,
      g); 
    loc += size;
    if loc > 800.0 {break}
  }

  let size = 16.0;
  let mut text = graphics::Text::new(16);
  text.color = [1.0, 0.0, 0.0, 1.0];
  let (px,py) = (600.0, size*1.5);
  let clock = "Time(ms): ".to_string() + &time.num_milliseconds().to_string();
  text.draw(
    &clock,
    f,
    &c.draw_state,
    c.trans(px, py).transform,
    g); 

}

fn render_cursor(c: graphics::context::Context, g: &mut GlGraphics, f: &mut GlyphCache, cc:CCs, t: &String) {
  graphics::clear([0.0, 0.0, 0.0, 1.0], g);

  //println!("{:?}", &t);
  let size = 48.0;
  let mut text = graphics::Text::new(48);
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
fn try_create_window(x: u32, y: u32) -> Result<GlutinWindow, String> {
  WindowSettings::new("ICEdit", [x, y])
    .exit_on_esc(true)
    .opengl(OPEN_GL)
    .build()
}

fn main() {

  //command-line
  let args = clap::App::new("IC_Edit")
    .version("0.2")
    .author("Kyle Headley <kyle.headley@colorado.edu>")
    .about("Incremental Text Editor")
    .args_from_usage(
      "-x --width=[width] 'editor width in pixels'
      -y --height=[height] 'editor height in pixels'")
    .subcommand(clap::SubCommand::with_name("test")
      .about("test options")
      .args_from_usage(
        "-x --width=[width] 'editor width in pixels'
        -y --height=[height] 'editor height in pixels'
        -s --rnd_start=[rnd_start] 'number of random starting commands'
        -a --rnd_adds=[rnd_adds] 'number of random commands after start'
        [auto_exit] -e --auto_exit 'exit the editor when all random commands are complete'")
    )
    .get_matches();
  //not the best usage of a subcommand, but ti works
  let test_args = if let Some(matches) = args.subcommand_matches("test") {matches} else {&args};
  let x = value_t!(test_args.value_of("width"), u32).unwrap_or(DEFAULT_WIDTH);
  let y = value_t!(test_args.value_of("height"), u32).unwrap_or(DEFAULT_HEIGHT);
  let rnd_start = value_t!(test_args.value_of("rnd_start"), u32).unwrap_or(DEFAULT_RND_START);
  let rnd_adds = value_t!(test_args.value_of("rnd_adds"), u32).unwrap_or(DEFAULT_RND_ADDITIONS);
  let auto_exit = test_args.is_present("auto_exit");

  //graphics
  let window = try_create_window(x, y).unwrap();
  let mut gl = GlGraphics::new(OPEN_GL);
  let exe_directory = current_exe().unwrap().parent().unwrap().to_owned();
  let mut font = GlyphCache::new(&exe_directory.join("../../FiraMono-Bold.ttf")).unwrap();

  //loop data
  let mut time = Duration::seconds(0);
  let mut needs_update = true;
  let mut command_key_down = false;
  let mut status = Inputstatus::Insert(Dir::R, false);
  let mut inputs = rnd_inputs(rnd_start);
  let more_inputs = rnd_inputs(rnd_adds);
  let mut more_inputs_iter = more_inputs.iter();
  let mut content_text = List::new().append("".to_string());

  for e in window.events().max_fps(60).ups(50) {
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
          //Key::LAlt |
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
          //Key::LAlt |
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
          /*Delete*/Key::D => {
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
          /*Undo*/Key::Z => {
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
          /*Redo*/Key::Y => {
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
          /*Switch*/ Key::H => {
            if command_key_down {
              let newstatus = match status {
                Inputstatus::Insert(_, _) | Inputstatus::Overwrite(_, _) => {
                  Inputstatus::EnterCursor(Box::new(status), CCs::Switch, List::new(), "".to_string())
                }
                Inputstatus::EnterCursor(_,_,_,_) => {
                  status
                }
              };
              status = newstatus;
              needs_update = true;
           }else{continue}
          } 
          /*Jmp*/ Key::J => {
            if command_key_down {
              let newstatus = match status {
                Inputstatus::Insert(_, _) | Inputstatus::Overwrite(_, _) => {
                  Inputstatus::EnterCursor(Box::new(status), CCs::Jmp, List::new(), "".to_string())
                }
                Inputstatus::EnterCursor(_,_,_,_) => {
                  status
                }
              };
              status = newstatus;
              needs_update = true;
           }else{continue}
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
          /*Show/hide cursors*/Key::S => {
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
      Event::Update(_) => {
        if !needs_update {
          match more_inputs_iter.next() {
            Some(cmd) => {
              inputs = inputs.append(cmd.clone());
              needs_update = true;
            }
            None => {
              if auto_exit {break}
            }
          }
        }
      }
      Event::Render(args) => {
        match status {
          Inputstatus::Insert(_, s) | Inputstatus::Overwrite(_, s) => {
            if needs_update {
              time = Duration::span(|| {
                content_text = build_content(&inputs, true, s);
                needs_update = false
              });
            }
            gl.draw(args.viewport(), |c, g| render(c, g, &mut font, &content_text, time));
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
