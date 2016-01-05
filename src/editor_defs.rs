use functional::List;

pub type Cursor = String;

#[derive(Debug, Clone)]
pub enum Dir {
  L,
  R,
}

impl Dir {
  pub fn opp(&self) -> Dir {
    match *self {
      Dir::L => {Dir::R}
      Dir::R => {Dir::L}
    }
  }
}

#[derive(Debug, Clone)]
pub enum Symbol {
	Cur(Cursor),
	Data(String),
}

#[derive(Debug, Clone)]
pub enum Action {
  Cmd(Command),
  Undo,
  Redo,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum CCs {
  Mk, Switch, Jmp, Join
}

pub type Zip<T> = (List<T>,List<T>);
pub type CZip<T> = (List<T>,Cursor,List<T>);

pub struct ViewParams {
	pub addcursor: bool,
	pub showcursors: bool
}

trait EditorPipeline {
    fn take_action(ac: Action) -> ();
    fn get_lines(vp: ViewParams) -> List<String>;
}