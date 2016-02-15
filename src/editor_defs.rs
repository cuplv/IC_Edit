use functional::List;
use time::Duration;
use std::fmt::Debug;
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

pub type Zip<T> = (List<T>,List<T>);
pub type CZip<T> = (List<T>,Cursor,List<T>);

pub struct ViewParams {
	pub addcursor: bool,
	pub showcursors: bool
}

pub trait CommonStats {
  fn time(self: &Self) -> Duration;
}
// impl<S: CommonStats> CommonStats for Box<S> {
//   fn time(self: &Self) -> Duration {
//     (**self).time()
//   }
// }
impl<'a, S: CommonStats> CommonStats for &'a S {
  fn time(self: &Self) -> Duration {
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

