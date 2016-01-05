
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

//Action list to undo buffer
pub fn al_to_ub(acts: &List<Action>) -> Zip<Command> {
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
pub fn passthrough(direction: Dir, before: List<Symbol>, after: List<Symbol>)
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

pub fn join_cursor(cur: Cursor, l: &List<Symbol>, r: &List<Symbol>)
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
pub fn cl_to_cz(commands: &List<Command>) -> CZip<Symbol> {
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
      Command::Switch(ref c) => {
        let withcursor = before.append(Symbol::Cur(ccursor.clone()));
        match join_cursor(c.clone(), &withcursor, &after) {
          Some((b,a)) => {(b, c.clone(), a)}
          None => {(before, ccursor, after)}
        }
      }
      Command::Jmp(ref c) => {
        match join_cursor(c.clone(), &before, &after) {
          Some((b,a)) => {
            let withcursor = b.append(Symbol::Cur(c.clone()));
            (withcursor, ccursor, a)
          }
          None => {(before, ccursor, after)}
        }
      }
      Command::Join(ref c) => {
        match join_cursor(c.clone(), &before, &after) {
          Some((b,a)) => {(b, c.clone(), a)}
          None => {(before, ccursor, after)}
        }
      }
    };

    before = before2;
    after = after2;
    ccursor = ccursor2;
  }

  (before, ccursor, after)
}

pub fn build_content(keys: &List<Action>) -> (List<Symbol>,List<Symbol>) {    
  let (commands, _) = al_to_ub(&keys.rev());
  let (before, _, after) = cl_to_cz(&commands.rev());    
  (before, after)
}