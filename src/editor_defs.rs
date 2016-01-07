use functional::List;

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

pub trait EditorPipeline {
    fn take_action(self: &mut Self, ac: Action) -> ();
    fn get_lines(self: &Self, vp: &ViewParams) -> List<String>;
}
