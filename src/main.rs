#![feature(zero_one)]

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
const DEFAULT_RND_CMDS: u32 = 0;
const DEFAULT_OUTFILE: Option<&'static str> = Some("testout.csv");

extern crate time;
extern crate rand;
#[macro_use]
extern crate clap;
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
use editor_defs::*;
use functional::List;
use spec::SpecEditor;
use fast::AdaptEditor;
use verifeditor::VerifEditor;
use randompie::RandomPie;

use adapton::adapton_sigs::Adapton;
use adapton::engine::Engine;
use adapton::naive::AdaptonFromScratch;

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
      Cmdtype::Join => panic!("impossible rnd command"),
      Cmdtype::Redo => {Action::Redo}
      Cmdtype::Undo => {Action::Undo}

    }
  }

  let mut current_user = 0;
  for _ in 0..num {
    match user_action(&mut rng, &dist) {
      a @ Action::Redo |
      a @ Action::Undo => {
        acts = acts.append(a.clone());
        acts = acts.append(a);      
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
    acts = acts.append(rnd_action(&mut rng, dist));
  }
  acts
}


// comment this to run interactive mode, uncomment for testing with a large stack
 fn main() {
   use std::thread;
   use std::thread::JoinHandle;
   let child =
     thread::Builder::new().stack_size(64 * 1024 * 1024).spawn(move || { main2() });
   child.unwrap().join();
 }

fn main2() {

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
      -d --rnd_dist [ins] [ovr] [rem] [mov] [goto] [make] [swch] [jump] [join] [redo] [undo] 'distribution integers for random commands'
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
        -d --rnd_dist [ins] [ovr] [rem] [mov] [goto] [make] [swch] [jump] [join] [redo] [undo] 'distribution integers for random commands'
        --start_seed=[start_seed]     'seed integer for random initial commands'
        --cmds_seed=[cmds_seed]       'seed integer for random additional commands'
        -u --users=[users]            'alternare rnd generation cycling between n cursors'
        -p --padding=[padding]        'initially separate the n cursors with [padding] lines each'
        -f --outfile=[outfile]        'filename for testing output'
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
        -d --rnd_dist [ins] [ovr] [rem] [mov] [goto] [make] [swch] [jump] [join] [redo] [undo] 'distribution integers for random commands'
        --start_seed=[start_seed]     'seed integer for random initial commands'
        --cmds_seed=[cmds_seed]       'seed integer for random additional commands'
        -u --users=[users]            'alternare rnd generation cycling between n cursors'
        -p --padding=[padding]        'initially separate the n cursors with [padding] lines each'
        -f --outfile=[outfile]        'filename for testing output'
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
    main_edit = Box::new(VerifEditor::<Engine,adapton::collection::List<Engine,Action>>::new(Engine::new(), rnd_inputs(rnd_start, start_seed, &rnd_dist, no_cursors)))
  } else if use_adapton {
    println!("Using AdaptEditor::<Engine,_> ...");
    main_edit = Box::new(AdaptEditor::<Engine,adapton::collection::List<Engine,Action>>::new(Engine::new(), rnd_inputs(rnd_start, start_seed, &rnd_dist, no_cursors)))
  } else if false {
    // Seems to overrun the stack;
    // tried using `export RUST_MIN_STACK=20485760` on the command line to mitigate this, but it didn't help.
    println!("Using AdaptEditor::<Naive,_> ...");
    main_edit = Box::new(AdaptEditor::<AdaptonFromScratch,adapton::collection::List<AdaptonFromScratch,Action>>::new(AdaptonFromScratch::new(), rnd_inputs(rnd_start, start_seed, &rnd_dist, no_cursors)))
  } else {
    println!("Using SpecEditor ...");
    main_edit = Box::new(SpecEditor::new(rnd_inputs(rnd_start, start_seed, &rnd_dist, no_cursors)));
  }

  // write csv file title
  match outfile {
    None => (),
    Some(ref mut f) => {
      if let Err(_) = writeln!(f, "##timestamp,user_tag,{}", main_edit.csv_title_line()) {
        panic!("can't write to file");
      }
    }
  }

  if windowless {
    loop {

      //update content

      let (_, csv) = main_edit.stats();
      match outfile {
        None => (),
        Some(ref mut f) => {
          if let Err(_) = writeln!(f, "{},{},{}", time::now().asctime(), test_tag, csv) {
            panic!("can't write to file");
          }
        }
      }

      //display stats
      {
        let (stat, _) = main_edit.stats();
        println!("Milliseconds: {}", (stat.time() as f64)/1000000.0);
      }
      //add action
      match more_inputs_iter.next() {
        Some(cmd) => {
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
  panic!("this version only runs in 'windowless' mode");
    }
}
