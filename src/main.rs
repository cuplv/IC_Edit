#![feature(zero_one)]

//-----------------
// This version uses a default (small) stack that
// can't handle larger documents. There is a hack
// near fn main to test with a large stack, but it
// can't show the graphics window at the same time
// future work should move the more demanding code
// into its own thread to solve this problem
//-----------------

// Default parameters
// ------------------

// icedit -x <Num> / --width <Num>
// Width in pixels
const DEFAULT_WIDTH: u32 = 800;

// icedit -y <Num> / --height <Num>
// Height in pixels
const DEFAULT_HEIGHT: u32 = 800;

// icedit -r / --reference
// uses spec implementation without optimisations

// icedit test
//see main() for full testing options
const TEST_WIDTH: u32 = 800;
const TEST_HEIGHT: u32 = 800;
const DEFAULT_RND_START: u32 = 0;
const DEFAULT_RND_CMDS: u32 = 1000;
const DEFAULT_OUTFILE: Option<&'static str> = Some("testout.csv");
const DEFAULT_SAMPLING: u32 = 11; //primes work best

extern crate time;
extern crate rand;
#[macro_use]
extern crate clap;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
#[macro_use]
extern crate adapton;

mod functional;
mod editor_defs;
mod spec;
mod fast;
mod verifeditor;
mod randompie;

use std::env::current_exe;
use std::fs::OpenOptions;
use std::io::prelude::*;
use rand::{Rng, StdRng, SeedableRng};
use time::Duration;
use glutin_window::GlutinWindow;
use graphics::Transformed;
use opengl_graphics::{GlGraphics, OpenGL};
use opengl_graphics::glyph_cache::GlyphCache;
use piston::event_loop::{Events, EventLoop};
use piston::input::{Button, Event, Input, Key};
use piston::window::WindowSettings;
use editor_defs::*;
use functional::List;
use spec::SpecEditor;
use fast::AdaptEditor;
use verifeditor::VerifEditor;
use randompie::RandomPie;

use adapton::adapton_sigs::Adapton;
use adapton::engine::Engine;
use adapton::naive::AdaptonFromScratch;

const OPEN_GL: OpenGL = OpenGL::V3_2;

enum Inputstatus {
  Insert(Dir, bool),    //Mode, direction, showcursors
  Overwrite(Dir, bool),
  EnterCursor(
    Box<Inputstatus>,   // prior input status
    CCs,                // current command in process
    SpecEditor,         // new cursor in progress
  )
}

fn firstline(l: &List<String>) -> String {
  l.head().unwrap_or(&"".to_string()).clone()
}

fn mem_usage() -> String {
  // python-based memory capture
  use std::process::Command;
  let output =
    Command::new("python")
    .arg("scripts/mem_use.py")
    .output()
    .unwrap_or_else(|e| { panic!("failed to execute python memory script: {}", e) });
  let outstr = String::from_utf8_lossy(&output.stdout);
  String::from(outstr.trim())
}

fn user_inputs(users: u32, pad: u32, num: u32, seed: Option<usize>, dist:&RandomPie) -> List<Action> {
  use rand::{Rng, StdRng, SeedableRng};
  let mut rng: StdRng = match seed {
    None => StdRng::new().unwrap(),
    Some(s) => {
      let s: &[_] = &[s];
      SeedableRng::from_seed(s)
    }
  };
  let dist = dist.no_cursors();
  let mut acts = List::new();

  for u in 1..users {
    acts = acts.append(Action::Cmd(Command::Mk(u.to_string())));
    for _ in 0..(pad){
      for _ in 0..20 {
        acts = acts.append(Action::Cmd(Command::Ins(" ".to_string(),Dir::R)));
      }
      acts = acts.append(Action::Cmd(Command::Ins("\n".to_string(),Dir::R)));
    }
  }

  fn user_action(rng: &mut StdRng, dist:&RandomPie) -> Action {
    let dir = rnd_dir(rng);
    match dist.get_cmd_type(rng) {
      Cmdtype::Ovr => {Action::Cmd(Command::Ovr(rnd_char(rng), dir))}
      Cmdtype::Ins => {Action::Cmd(Command::Ins(rnd_char(rng), dir))}
      Cmdtype::Rem => {Action::Cmd(Command::Rem(dir))}
      Cmdtype::Mov => {Action::Cmd(Command::Move(dir))}
      Cmdtype::Goto => {Action::Cmd(Command::Goto(rng.gen()))}
      Cmdtype::Make |
      Cmdtype::Swch |
      Cmdtype::Jump |
      Cmdtype::Join => panic!("impossible rnd command join"),
      Cmdtype::Redo => panic!("impossible rnd command redo"),
      Cmdtype::Undo => {Action::Undo}

    }
  }

  let mut current_user = 0;
  let undos = dist.undo_count();
  for _ in 0..num {
    match user_action(&mut rng, &dist) {
      Action::Redo => panic!("impossible rnd command redo"),
      Action::Undo => {
        for _ in 0..undos {
          acts = acts.append(Action::Undo);
        }
        if rng.gen() {
          let redos = undos / 2;
          for _ in 0..redos {
            acts = acts.append(Action::Redo);
          }
        }
      }
      a @ _ => {
        acts = acts.append(Action::Cmd(Command::Switch(current_user.to_string())));
        acts = acts.append(a);      
      }
    }
    current_user = (current_user + 1) % users;
  }
  acts
}

fn rnd_char(rng: &mut StdRng) -> String {
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

fn rnd_dir(rng: &mut StdRng) -> Dir {
  if rng.gen_range(0,10) > 3 {Dir::R} else {Dir::L}
}

fn rnd_inputs(num: u32, seed: Option<usize>, dist:&RandomPie, nc: bool) -> List<Action> {
  let mut rng: StdRng = match seed {
    None => StdRng::new().unwrap(),
    Some(s) => {
      let s: &[_] = &[s];
      SeedableRng::from_seed(s)
    }
  };
  let mut cursor_count = 1;
  let mut acts = List::new();

  fn rnd_cursor(rng: &mut StdRng, max_cursor: u32) -> Cursor {
    let rn: u32 = rng.gen_range(0, max_cursor);
    rn.to_string()
  };

  let mut rnd_action = |rng: &mut StdRng, dist:&RandomPie| {
    let cmd = match nc {
      false => dist.get_cmd_type(rng),
      true => dist.no_cursors().get_cmd_type(rng)
    };
    match cmd {
      Cmdtype::Ovr => {Action::Cmd(Command::Ovr(rnd_char(rng), rnd_dir(rng)))}
      Cmdtype::Ins => {Action::Cmd(Command::Ins(rnd_char(rng), rnd_dir(rng)))}
      Cmdtype::Rem => {Action::Cmd(Command::Rem(rnd_dir(rng)))}
      Cmdtype::Mov => {Action::Cmd(Command::Move(rnd_dir(rng)))}
      Cmdtype::Goto => {Action::Cmd(Command::Goto(rng.gen()))}
      Cmdtype::Make => {
        cursor_count = cursor_count + 1;
        Action::Cmd(Command::Mk((cursor_count - 1).to_string()))
      }
      Cmdtype::Swch => {Action::Cmd(Command::Switch(rnd_cursor(rng, cursor_count)))}
      Cmdtype::Jump => {Action::Cmd(Command::Jmp(rnd_cursor(rng, cursor_count)))}
      Cmdtype::Join => {Action::Cmd(Command::Join(rnd_cursor(rng, cursor_count)))}
      Cmdtype::Redo => {Action::Redo}
      Cmdtype::Undo => {Action::Undo}

    }
  };

  for _ in 0..num {
    match rnd_action(&mut rng, dist) {
      Action::Undo => {
        let undos = dist.undo_count();
        for _ in 0..undos {
          acts = acts.append(Action::Undo);
        }
        if rng.gen() {
          let redos = undos / 2;
          for _ in 0..redos {
            acts = acts.append(Action::Redo);
          }
        }
      }
      a @ _ => acts = acts.append(a)
    }
  }
  acts
}

fn render(c: graphics::context::Context, g: &mut GlGraphics, f: &mut GlyphCache, t: &List<String>, time: u64, s: &Inputstatus) {
  graphics::clear([0.0, 0.0, 0.0, 1.0], g);

  //main text
  let size = 22.0;
  let mut text = graphics::Text::new(22);
  text.color = [1.0, 1.0, 1.0, 1.0];
  let mut loc = size;

  for st in t.iter() {
    text.draw(
      st, f, &c.draw_state,
      c.trans(10.0, loc).transform,
      g); 
    loc += size;
    if loc > 800.0 {break}
  }

  //info section
  let hud_bcolor = [0.02, 0.02, 0.02, 0.8];
  let hud_back = [590.0, 5.0, 200.0, 90.0];
  graphics::rectangle(hud_bcolor, hud_back, c.transform, g); // shaded background
  let size = 16.0;
  let mut text = graphics::Text::new(16);
  text.color = [0.5, 1.0, 0.5, 1.0];
  let (px,py) = (600.0, size*1.5);
  let clock = "Time(ms): ".to_string() + &((time as f32)/1000000 as f32).to_string();
  text.draw(
    &clock, f, &c.draw_state,
    c.trans(px, py).transform,
    g);
  let (stat, cur) = match *s {
    Inputstatus::Insert(ref d,ref show) => {
      let stat = "I ";
      let dir = match d { &Dir::L => "<- (c-arrows)", &Dir::R => "-> (c-arrows)" };
      let cur = if *show { "Visible (c-s)"} else { "Invisible (c-s)"};
      (stat.to_string() + &dir, cur.to_string())
    }
    Inputstatus::Overwrite(ref d,ref show) => {
      let stat = "O ";
      let dir = match d { &Dir::L => "<- (c-arrows)", &Dir::R => "-> (c-arrows)" };
      let cur = if *show { "Visible (c-s)"} else { "Invisible (c-s)"};
      (stat.to_string() + &dir, cur.to_string())
    }
    _ => ("".to_string(), "".to_string()) // should not happen
  };
  text.draw(
    &stat, f, &c.draw_state,
    c.trans(px, py+(size*1.5)).transform,
    g); 
  text.draw(
    &cur, f, &c.draw_state,
    c.trans(px, py+(size*3.0)).transform,
    g); 

}

fn render_cursor(c: graphics::context::Context, g: &mut GlGraphics, f: &mut GlyphCache, cc:CCs, t: &String) {
  graphics::clear([0.0, 0.0, 0.0, 1.0], g);

  //choose title
  let size = 48.0;
  let mut text = graphics::Text::new(48);
  text.color = [1.0, 1.0, 1.0, 1.0];
  let (px,py) = (200.0,250.0);
  let prompt = match cc {
    CCs::Goto => {"Goto item number: "}
    CCs::Mk => {"Create cursor: "}
    CCs::Switch => {"Switch to cursor: "}
    CCs::Jmp => {"Jump to cursor: "}
    CCs::Join => {"Join with cursor: "}
  }.to_string();

  //render screen
  text.draw(
    &prompt, f, &c.draw_state,
    c.trans(px, py).transform,
    g); 
  text.draw(
    t, f, &c.draw_state,
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

// comment this to run interactive mode, uncomment for testing with a large stack
// fn main() {
//   use std::thread;
//   use std::thread::JoinHandle;
//   let child =
//     thread::Builder::new().stack_size(64 * 1024 * 1024).spawn(move || { main2() });
//   child.unwrap().join();
// }

fn main() {

  //command-line
  let args = clap::App::new("IC_Edit")
    .version("0.2")
    .author("Kyle Headley <kyle.headley@colorado.edu>")
    .about("Incremental Text Editor")
    .args_from_usage("\
      -x --width=[width]              'initial editor width in pixels'
      -y --height=[height]            'initial editor height in pixels'
      -s --rnd_start=[rnd_start]      'number of random starting commands'
      -c --rnd_cmds=[rnd_cmds]        'number of random commands after start'
      -d --rnd_dist [ins] [ovr] [rem] [mov] [goto] [make] [swch] [jump] [join] [undo] [undocount] 'distribution integers for random commands'
      --start_seed=[start_seed]       'seed integer for random initial commands'
      --cmds_seed=[cmds_seed]         'seed integer for random additional commands'
      -h --hide_curs                  'hide cursors initially'
      [spec_only] -r --reference      'disable Adapton optimizations' ")
    .subcommand(clap::SubCommand::with_name("test")
      .about("test options")
      .args_from_usage("\
        -x --width=[width]            'editor width in pixels'
        -y --height=[height]          'editor height in pixels'
        -s --rnd_start=[rnd_start]    'number of random starting commands'
        -c --rnd_cmds=[rnd_cmds]      'number of random commands after start'
        -d --rnd_dist [ins] [ovr] [rem] [mov] [goto] [make] [swch] [jump] [join] [undo] [undocount] 'distribution integers for random commands'
        --start_seed=[start_seed]     'seed integer for random initial commands'
        --cmds_seed=[cmds_seed]       'seed integer for random additional commands'
        --sparse=[sparse]             'distance between arts'
        -u --users=[users]            'alternare rnd generation cycling between n cursors'
        -p --padding=[padding]        'initially separate the n cursors with [padding] lines each'
        -f --outfile=[outfile]        'filename for testing output'
        --samp=[samp]                 'downsample mempry collection'
        -t --test_tag=[test_tag]      'user-defined id info for the results csv'
        -h --hide_curs                'hide cursors initially'
        -n --no_cursors               'do not use cursors in random commands'
        [spec_only] -r --reference    'only test reference implementation'
        [fast_only] -a --adapton      'only test adapton implementation'
        -o --keep_open                'do not exit the editor when testing is complete' ")
    )
    .subcommand(clap::SubCommand::with_name("windowless")
      .about("windowless options")
      .args_from_usage("\
        -s --rnd_start=[rnd_start]    'number of random starting commands'
        -c --rnd_cmds=[rnd_cmds]      'number of random commands after start'
        -d --rnd_dist [ins] [ovr] [rem] [mov] [goto] [make] [swch] [jump] [join] [undo] [undocount] 'distribution integers for random commands'
        --start_seed=[start_seed]     'seed integer for random initial commands'
        --cmds_seed=[cmds_seed]       'seed integer for random additional commands'
        --sparse=[sparse]             'distance between arts'
        -u --users=[users]            'alternare rnd generation cycling between n cursors'
        -p --padding=[padding]        'initially separate the n cursors with [padding] lines each'
        -f --outfile=[outfile]        'filename for testing output'
        --samp=[samp]                 'downsample mempry collection'
        -t --test_tag=[test_tag]      'user-defined id info for the results csv'
        -h --hide_curs                'hide cursors initially'
        -n --no_cursors               'do not use cursors in random commands'
        [spec_only] -r --reference    'only test reference implementation'
        [fast_only] -a --adapton      'only test adapton implementation' ")
    )
    .get_matches();
  let test;
  let windowless;
  let test_args =
    if let Some(matches) = args.subcommand_matches("test") {
      test = true; windowless = false; matches
    } else if let Some(matches) = args.subcommand_matches("windowless") {
      test = true; windowless = true; matches
    } else {
      test = false; windowless = false; &args
    };
  let x = value_t!(test_args.value_of("width"), u32).unwrap_or(if test {TEST_WIDTH} else {DEFAULT_WIDTH});
  let y = value_t!(test_args.value_of("height"), u32).unwrap_or(if test {TEST_HEIGHT} else {DEFAULT_HEIGHT});
  let rnd_start = value_t!(test_args.value_of("rnd_start"), u32).unwrap_or(if test {DEFAULT_RND_START} else {0});
  let rnd_adds = value_t!(test_args.value_of("rnd_cmds"), u32).unwrap_or(if test {DEFAULT_RND_CMDS} else {0});
  let start_seed = match value_t!(test_args.value_of("start_seed"), usize).unwrap_or(0) { 0 => None, v => Some(v)};
  let cmds_seed = match value_t!(test_args.value_of("cmds_seed"), usize).unwrap_or(0) { 0 => None, v => Some(v)};
  let rnd_dist = values_t!(test_args.values_of("rnd_dist"), u32).unwrap_or(vec![50, 20, 10, 20, 1, 1, 1, 1, 1, 1, 1]);
  let rnd_dist = RandomPie::new(rnd_dist);
  let users = value_t!(test_args.value_of("users"), u32).ok();
  let padding = value_t!(test_args.value_of("padding"), u32).unwrap_or(0);
  let sparse = value_t!(test_args.value_of("sparse"), usize).unwrap_or(3);
  let rnd_adds = match users { None => rnd_adds, Some(_) => rnd_adds/2 };
  let keep_open = if test {test_args.is_present("keep_open")} else {true};
  let show_curs = !test_args.is_present("hide_curs");
  let no_cursors = test_args.is_present("no_cursors");
  let use_adapton = !test_args.is_present("spec_only");
  let use_spec = !test_args.is_present("fast_only");
  let test_tag = match test_args.value_of("test_tag") {
    None => "default",
    Some(t) => t
  };
  let outfile = match test_args.value_of("outfile") {
    None => if test {DEFAULT_OUTFILE} else {None},
    Some(f) => Some(f)
  };
  let mut outfile = outfile.map(|f| {
    OpenOptions::new()
    .create(true)
    .write(true)
    .append(true)
    .open(f)
    .unwrap()
  });
  let downsample = value_t!(test_args.value_of("samp"), u32).unwrap_or(DEFAULT_SAMPLING);
  let mut action_count = 0;
  let mut last_mem = "0".to_string();

  // this is being replaced by adapton logging
  let mut logfile = &mut
    OpenOptions::new()
    .create(true)
    .write(true)
    .append(true)
    .open("icedit_log.gmv")
    .unwrap();

  //TODO: the clap library supports this in param parsing
  //assert_eq!(use_adapton || use_spec, true);

  //loop data
  let mut main_edit: Box<EditorPipeline>;
  let mut needs_update = true;
  let more_inputs = if let Some(u) = users {
    user_inputs(u, padding, rnd_adds, cmds_seed, &rnd_dist).rev()
  } else {
    rnd_inputs(rnd_adds, cmds_seed, &rnd_dist, no_cursors).rev()
  };
  let mut more_inputs_iter = more_inputs.iter();
  let mut content_text = List::new().append("".to_string());

  //select editor  
  if test && use_adapton && use_spec {
    //println!("Preparing to perform dynamic verification ...");
    println!("Using VerifEditor::<Engine,_> ...");
    main_edit = Box::new(VerifEditor::<Engine,adapton::collection::List<Engine,Action>>::new(Engine::new(), rnd_inputs(rnd_start, start_seed, &rnd_dist, no_cursors), sparse))
  } else if use_adapton {
    println!("Using AdaptEditor::<Engine,_> ...");
    main_edit = Box::new(AdaptEditor::<Engine,adapton::collection::List<Engine,Action>>::new(Engine::new(), rnd_inputs(rnd_start, start_seed, &rnd_dist, no_cursors), sparse))
  } else if false {
    // Seems to overrun the stack;
    // tried using `export RUST_MIN_STACK=20485760` on the command line to mitigate this, but it didn't help.
    println!("Using AdaptEditor::<Naive,_> ...");
    main_edit = Box::new(AdaptEditor::<AdaptonFromScratch,adapton::collection::List<AdaptonFromScratch,Action>>::new(AdaptonFromScratch::new(), rnd_inputs(rnd_start, start_seed, &rnd_dist, no_cursors), sparse))
  } else {
    println!("Using SpecEditor ...");
    main_edit = Box::new(SpecEditor::new(rnd_inputs(rnd_start, start_seed, &rnd_dist, no_cursors)));
  }

  // write csv file title
  match outfile {
    None => (),
    Some(ref mut f) => {
      if let Err(_) = writeln!(f, "##timestamp,user_tag,{},os_MB_used", main_edit.csv_title_line()) {
        panic!("can't write to file");
      }
    }
  }

  if windowless {
    loop {

      //update content and display stats
      {
        if action_count % downsample == 0 {last_mem = format!("{}",mem_usage())};
        let (stat, csv) = main_edit.stats();
        match outfile {
          None => (),
          Some(ref mut f) => {
            if let Err(_) = writeln!(f, "{},{},{},{}", time::now().asctime(), test_tag, csv, last_mem) {
              panic!("can't write to file");
            }
          }
        };
        println!("Milliseconds: {}", (stat.time() as f64)/1000000.0);
      }

      //add action
      match more_inputs_iter.next() {
        Some(cmd) => {
          action_count += 1;
          main_edit.take_action(cmd.clone(), None);
          main_edit.get_lines(&ViewParams{
            addcursor: true,
            showcursors: false
          }, None);
        }
        None => {
          break
        }
      }


    }
  }else{
    // graphics
    let window = try_create_window(x, y).unwrap();
    let mut gl = GlGraphics::new(OPEN_GL);
    let exe_directory = current_exe().unwrap().parent().unwrap().to_owned();
    let mut font = GlyphCache::new(&exe_directory.join("../../FiraMono-Bold.ttf")).unwrap();
    
    // input
    let mut command_key_down = false;
    let mut status = Inputstatus::Insert(Dir::R, show_curs);

    for e in window.events().max_fps(60).ups(50) {
      match e {
        //gives typed char or empty
        Event::Input(Input::Text(t)) => {
          if t == "" || command_key_down || t.len() > 1 {continue}
          status = match status {
            Inputstatus::Insert(d, s) => {
              main_edit.take_action(Action::Cmd(Command::Ins(t,d.clone())), None);
              Inputstatus::Insert(d, s)
            }
            Inputstatus::Overwrite(d, s) => {
              main_edit.take_action(Action::Cmd(Command::Ovr(t,d.clone())), None);
              Inputstatus::Overwrite(d, s)
            }
            Inputstatus::EnterCursor(p,c,mut e @ _) => {
              e.take_action(Action::Cmd(Command::Ins(t,Dir::R)), None);
              Inputstatus::EnterCursor(p,c,e)
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
                  Inputstatus::EnterCursor(p,c,a) => {
                    Inputstatus::EnterCursor(p,c,a)                  
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
                  Inputstatus::EnterCursor(p,c,a) => {
                    Inputstatus::EnterCursor(p,c,a)                  
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
                  Inputstatus::EnterCursor(p,c,e) => {
                    Inputstatus::EnterCursor(p,c,e)                  
                  }
                };
              }
              else{
                status = match status {
                  Inputstatus::Insert(d, s) => {
                    main_edit.take_action(
                      Action::Cmd(Command::Move(Dir::L)), None
                    );
                    Inputstatus::Insert(d, s)
                  }
                  Inputstatus::Overwrite(d, s) => {
                    main_edit.take_action(
                      Action::Cmd(Command::Move(Dir::L)), None
                    );
                    Inputstatus::Overwrite(d, s)
                  }
                  Inputstatus::EnterCursor(p,c,mut e @ _) => {
                    e.take_action(Action::Cmd(Command::Move(Dir::L)), None);
                    Inputstatus::EnterCursor(p,c,e)                  
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
                  Inputstatus::EnterCursor(p,c,e) => {
                    Inputstatus::EnterCursor(p,c,e)                  
                  }
                };
              }
              else{
                status = match status {
                  Inputstatus::Insert(d, s) => {
                    main_edit.take_action(
                      Action::Cmd(Command::Move(Dir::R)), None
                    );
                    Inputstatus::Insert(d, s)
                  }
                  Inputstatus::Overwrite(d, s) => {
                    main_edit.take_action(
                      Action::Cmd(Command::Move(Dir::R)), None
                    );
                    Inputstatus::Overwrite(d, s)
                  }
                  Inputstatus::EnterCursor(p,c,mut e @ _) => {
                    e.take_action(Action::Cmd(Command::Move(Dir::R)), None);
                    Inputstatus::EnterCursor(p,c,e)                  
                  }
                };
                needs_update = true;
              }
            }
            /*Delete*/Key::D => {
              if command_key_down {
                status = match status {
                  Inputstatus::Insert(d, s) => {
                    main_edit.take_action(
                      Action::Cmd(Command::Rem(d.clone())), None
                    );
                    Inputstatus::Insert(d, s)
                  }
                  Inputstatus::Overwrite(d, s) => {
                    main_edit.take_action(
                      Action::Cmd(Command::Rem(d.clone())), None
                    );
                    Inputstatus::Overwrite(d, s)
                  }
                  Inputstatus::EnterCursor(p,c,mut e @ _) => {
                    e.take_action(Action::Cmd(Command::Rem(Dir::R)), None);
                    Inputstatus::EnterCursor(p,c,e)                  
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
                    main_edit.take_action(
                      Action::Cmd(Command::Rem(d.opp())), None
                    );
                    Inputstatus::Insert(d, s) 
                  }
                  Inputstatus::Overwrite(d, s) => {
                    main_edit.take_action(
                      Action::Cmd(Command::Rem(d.opp())), None
                    );
                    Inputstatus::Overwrite(d, s) 
                  }
                  Inputstatus::EnterCursor(p,c,mut e @ _) => {
                    e.take_action(Action::Cmd(Command::Rem(Dir::L)), None);
                    Inputstatus::EnterCursor(p,c,e)                  
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
                    main_edit.take_action(
                      Action::Cmd(Command::Ins("\n".to_string(), d.clone())), None
                    );
                    Inputstatus::Insert(d, s)
                  }
                  Inputstatus::Overwrite(d, s) => {
                    main_edit.take_action(
                      Action::Cmd(Command::Ins("\n".to_string(), d.clone())), None
                    );
                    Inputstatus::Overwrite(d, s)
                  }
                  Inputstatus::EnterCursor(p,c,mut e @ _) => {
                    let content = firstline(&e.get_lines(&ViewParams{addcursor: false, showcursors: false}, None));
                    if let CCs::Goto = c {
                      if let Ok(pos) = content.parse() {
                        main_edit.take_action(Action::Cmd(Command::Goto(pos)), None);
                      }
                    } else {
                      let newcommand = match c {
                          CCs::Goto => { unreachable!() } // selected by if let above
                          CCs::Mk => {Command::Mk(content)}
                          CCs::Switch => {Command::Switch(content)}
                          CCs::Jmp => {Command::Jmp(content)}
                          CCs::Join => {Command::Join(content)}
                      };
                      main_edit.take_action(Action::Cmd(newcommand), None);
                    }
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
                    main_edit.take_action(Action::Undo, None);
                    status
                  }
                  Inputstatus::EnterCursor(p,cc,mut e @ _) => {
                    e.take_action(Action::Undo, None);
                    Inputstatus::EnterCursor(p,cc,e)
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
                    main_edit.take_action(Action::Redo, None);
                    status
                  }
                  Inputstatus::EnterCursor(p,cc,mut e @ _) => {
                    e.take_action(Action::Redo, None);
                    Inputstatus::EnterCursor(p,cc,e)
                  }
                };
                status = newstatus;
                needs_update = true;
              } else {continue}
            }
            /*Goto*/ Key::G => {
              if command_key_down{
                  let newstatus = match status {
                    Inputstatus::Insert(_, _) | Inputstatus::Overwrite(_, _) => {
                      Inputstatus::EnterCursor(Box::new(status), CCs::Goto, SpecEditor::new(List::new()))
                    }
                    Inputstatus::EnterCursor(_,_,_) => {
                      status
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
                      Inputstatus::EnterCursor(Box::new(status), CCs::Mk, SpecEditor::new(List::new()))
                    }
                    Inputstatus::EnterCursor(_,_,_) => {
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
                    Inputstatus::EnterCursor(Box::new(status), CCs::Switch, SpecEditor::new(List::new()))
                  }
                  Inputstatus::EnterCursor(_,_,_) => {
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
                    Inputstatus::EnterCursor(Box::new(status), CCs::Jmp, SpecEditor::new(List::new()))
                  }
                  Inputstatus::EnterCursor(_,_,_) => {
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
                    Inputstatus::EnterCursor(Box::new(status), CCs::Join, SpecEditor::new(List::new()))
                  }
                  Inputstatus::EnterCursor(_,_,_) => {
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
                    Inputstatus::EnterCursor(p,c,e) => {
                      Inputstatus::EnterCursor(p,c,e)
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
                main_edit.take_action(cmd.clone(), None);
                needs_update = true;
              }
              None => {
                if !keep_open {break}
              }
            }
          }
        }
        Event::Render(args) => {
          match status {
            Inputstatus::Insert(_, s) | Inputstatus::Overwrite(_, s) => {
              if needs_update {
                action_count += 1;
                content_text = main_edit.get_lines(&ViewParams{
                  addcursor: true,
                  showcursors: s
                }, Some(logfile));
                let (_, csv) = main_edit.stats();
                if action_count % downsample == 0 {last_mem = format!("{}",mem_usage())};
                match outfile {
                  None => (),
                  Some(ref mut f) => {
                    if let Err(_) = writeln!(f, "{},{},{},{}", time::now().asctime(), test_tag, csv, last_mem) {
                      panic!("can't write to file");
                    }
                  }
                };
                needs_update = false
              }
              let (stat, _) = main_edit.stats();            
              gl.draw(args.viewport(), |c, g| render(c, g, &mut font, &content_text, stat.time(), &status));
            }
            Inputstatus::EnterCursor(_, ref cc, ref mut e @ _) => {
              let ct = firstline(&e.get_lines(&ViewParams{ addcursor: true, showcursors: false }, None));
              gl.draw(args.viewport(), |c, g| render_cursor(c, g, &mut font, cc.clone(), &ct));
            }
          }
          
        }
        _ => {}

      }
    }

  }
  std::process::exit(0);
}
