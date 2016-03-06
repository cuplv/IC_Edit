use functional::List;
use time::{self, Duration};
use std::fmt::{Debug, Display};
use std::fmt;
use std::fs::File;

pub type Cursor = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Dir {
  L,
  R,
}

//type Dir = Dir2

impl Dir {
  pub fn opp(&self) -> Dir {
    match *self {
      Dir::L => {Dir::R}
      Dir::R => {Dir::L}
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Symbol {
	Cur(Cursor),
	Data(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
  Cmd(Command),
  Undo,
  Redo,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Command {
  Ins(String, Dir),   //Insert <String>, moving cursor <Dir>
  Rem(Dir),           //Remove character located <Dir>
  Move(Dir),          //Move cursor <Dir>
  Ovr(String, Dir),   //Overwrite with <String>, moving cursor <Dir>
  Mk(Cursor),
  Switch(Cursor),
  Jmp(Cursor),
  Join(Cursor),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cmdtype {
  Ins, Ovr, Rem, Mov,
  Make, Swch, Jump, Join,
  Undo, Redo
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CCs {
  Mk, Switch, Jmp, Join
}

impl Action {
  pub fn simple_view(&self) -> &str {
    match *self {
      Action::Undo => "Undo",
      Action::Redo => "Redo",
      Action::Cmd(ref c) => c.simple_view()
    }
  }
}

impl Command {
  pub fn simple_view(&self) -> &str {
    match *self {
      Command::Ins(_,_) => "Insert",
      Command::Rem(_) => "Remove",
      Command::Move(_) => "Move",
      Command::Ovr(_,_) => "Overwrite",
      Command::Mk(_) => "MakeCursor",
      Command::Switch(_) => "Switch",
      Command::Jmp(_) => "JumpTo",
      Command::Join(_) => "Join",
    }
  }
}

impl Display for Action {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.simple_view())
  }
}

impl Display for Command {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.simple_view())
  }
}

pub type Zip<T> = (List<T>,List<T>);
pub type CZip<T> = (List<T>,Cursor,List<T>);

pub struct ViewParams {
	pub addcursor: bool,
	pub showcursors: bool
}

pub trait CommonStats {
  fn time(self: &Self) -> u64;
}
// impl<S: CommonStats> CommonStats for Box<S> {
//   fn time(self: &Self) -> Duration {
//     (**self).time()
//   }
// }
impl<'a, S: CommonStats> CommonStats for &'a S {
  fn time(self: &Self) -> u64 {
    (**self).time()
  }
}

pub trait EditorPipeline {
  fn take_action(self: &mut Self, ac: Action, log: Option<&mut File>) -> ();
  fn get_lines(self: &mut Self, vp: &ViewParams, log: Option<&mut File>) -> List<String>;
  fn csv_title_line(self: &Self) -> String { "Unspecified Data:".to_string() }
  fn stats(self: &mut Self) -> (
    &CommonStats, // formal Stats
    String            // some CSV string
  );
}

pub trait StatProvider<S: CommonStats> {
  fn all_stats(self: &mut Self) -> S; 
}

pub fn measure_ns<F:FnOnce()>(f: F) -> u64 {
  let start = time::precise_time_ns();
  f();
  let end = time::precise_time_ns();
  end - start
}



