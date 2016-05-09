use functional::List;
use time::Duration;
use editor_defs::*;
use std::fmt::Debug;
use std::fs::File;

#[derive(Debug)]
pub struct SpecStats {
  gen_time: u64,
  processed_cmds: u32,
  search_failures: u32,
  read_past_ends: u32,
}

impl SpecStats {
  pub fn new() -> SpecStats {
    SpecStats{
      gen_time: 0,
      processed_cmds: 0,
      search_failures: 0,
      read_past_ends: 0,
    }
  }
}
impl CommonStats for SpecStats {
  fn time(self: &Self) -> u64 {
    self.gen_time
  }
}

pub struct SpecEditor {
    next_id: usize,
    total_actions: usize,
    last_stats: SpecStats,
    last_action: Option<Action>,
    actions: List<Action>,
}

impl SpecEditor {
  pub fn new(initial_actions: List<Action>) -> SpecEditor {
    SpecEditor{
      next_id: 0,
      total_actions: initial_actions.iter().count(),
      last_stats: SpecStats::new(),
      last_action: None,
      actions: initial_actions.rev(),
    }
  }
}

//Action list to undo buffer
pub fn undobuff_of_actions(acts: &List<Action>, mut stat: SpecStats) -> (Zip<Command>, SpecStats) {
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
  ((content,buffer), stat)
}

//move the zipper through cursors in the given direction 
pub fn passthrough(direction: Dir, before: List<Symbol>, after: List<Symbol>, mut stat: SpecStats)
  -> ((List<Symbol>, List<Symbol>), SpecStats) {
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
    Dir::L => {((first, second), stat)}
    Dir::R => {((second, first), stat)}
  }
}

pub fn goto_item(loc: isize, l: &List<Symbol>, r: &List<Symbol>, mut stat: SpecStats)
-> ((List<Symbol>, List<Symbol>), SpecStats) {
  let mut count = 0;

  //count right
  let mut second = r.clone();
  loop {
    match second.head().map(|h| h.clone()) {
      None => {break}
      Some(Symbol::Data(d)) => {
        if d == "\n" {} else {count += 1};
        second = second.tail();
      }
      Some(Symbol::Cur(c)) => {
        second = second.tail();
      }
    }
  }

  //count left and shift
  let mut first = l.clone();
  let mut second = r.clone();
  loop {
    match first.head().map(|h| h.clone()) {
      None => {break}
      Some(Symbol::Data(d)) => {
        if d == "\n" {} else {count += 1};
        first = first.tail();
        second = second.append(Symbol::Data(d));
      }
      Some(Symbol::Cur(c)) => {
        first = first.tail();
        second = second.append(Symbol::Cur(c));
      }
    }
  }

  // calculate location
  count += 1;
  let loc = ((loc % count) + count) % count; // mod operator so that -1 = last item

  // seek to location
  let mut count = 0;
  let mut first = List::new();
  if loc == 0 {return ((first, second), stat)};
  loop {
    match second.head().map(|h| h.clone()) {
      None => { unreachable!() } // we counted the length
      Some(Symbol::Data(d)) => {
        if d == "\n" {} else {count += 1};
        second = second.tail();
        first = first.append(Symbol::Data(d));
        if count == loc {break};
      }
      Some(Symbol::Cur(c)) => {
        second = second.tail();
        first = first.append(Symbol::Cur(c));
      }
    }
  }
  ((first, second), stat)
}

pub fn join_cursor(cur: Cursor, l: &List<Symbol>, r: &List<Symbol>, mut stat: SpecStats)
-> (Option<(List<Symbol>, List<Symbol>)>, SpecStats) {
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
          return (Some((first.tail(),second)), stat)
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
      None => {return (None, stat)}
      Some(Symbol::Data(d)) => {
        second = second.tail();
        first = first.append(Symbol::Data(d));
      }
      Some(Symbol::Cur(c)) => {
        if cur == c {
          return (Some((first,second.tail())), stat)
        }else{
          second = second.tail();
          first = first.append(Symbol::Cur(c));
        }
      }
    }
  }

}


// command list to content zipper
pub fn content_of_commands(commands: &List<Command>, mut stat: SpecStats) -> (CZip<Symbol>, SpecStats) {
  let mut before: List<Symbol> = List::new();
  let mut ccursor: Cursor = "0".to_string();
  let mut after: List<Symbol> = List::new();

  for command in commands.iter() {
    let ((before2, ccursor2, after2), s) =
    match *command {
      Command::Ins(ref d, Dir::R) => {
        let ((b, a), s) = passthrough(Dir::R, before, after, stat);
        ((b.append(Symbol::Data(d.clone())), ccursor, a), s)
      }
      Command::Ins(ref d, Dir::L) => {
        let ((b, a),s) = passthrough(Dir::L, before, after, stat);
        ((b, ccursor, a.append(Symbol::Data(d.clone()))),s)
      }
      Command::Rem(Dir::L) => {
        let ((b, a),s) = passthrough(Dir::L, before, after, stat);
        ((b.tail(), ccursor, a),s)
      }
      Command::Rem(Dir::R) => {
        let ((b, a),s) = passthrough(Dir::R, before, after, stat);
        ((b, ccursor, a.tail()),s)
      }
      Command::Move(Dir::L) => {
        let ((b, a),s) = passthrough(Dir::L, before, after, stat);
        match b.head() {
          None => {((List::new(), ccursor, a),s)}
          Some(d) => {((b.tail(), ccursor, a.append(d.clone())),s)}
        }
      }
      Command::Move(Dir::R) => {
        let ((b, a),s) = passthrough(Dir::R, before, after, stat);
        match a.head() {
          None => {((b, ccursor, List::new()),s)}
          Some(d) => {((b.append(d.clone()), ccursor, a.tail()),s)}
        }
      }
      Command::Goto(pos) => {
        let ((b, a), s) = goto_item(pos, &before, &after, stat);
        ((b, ccursor, a),s)
      }
      Command::Ovr(ref d, Dir::L) => {
        let ((b, a),s) = passthrough(Dir::L, before, after, stat);
        ((b.tail(), ccursor, a.append(Symbol::Data(d.clone()))),s)
      }
      Command::Ovr(ref d, Dir::R) => {
        let ((b, a),s) = passthrough(Dir::R, before, after, stat);
        ((b.append(Symbol::Data(d.clone())), ccursor, a.tail()),s)
      }
      Command::Mk(ref c) => {
        ((before.append(Symbol::Cur(c.clone())), ccursor, after),stat)
      }
      Command::Switch(ref c) => {
        let withcursor = before.append(Symbol::Cur(ccursor.clone()));
        let (find,s) = join_cursor(c.clone(), &withcursor, &after, stat);
        match find {
          Some((b,a)) => {((b, c.clone(), a),s)}
          None => {((before, ccursor, after),s)}
        }
      }
      Command::Jmp(ref c) => {
        match join_cursor(c.clone(), &before, &after, stat) {
          (Some((b,a)),s) => {
            let withcursor = b.append(Symbol::Cur(c.clone()));
            ((withcursor, ccursor, a),s)
          }
          (None,s) => {((before, ccursor, after),s)}
        }
      }
      Command::Join(ref c) => {
        match join_cursor(c.clone(), &before, &after, stat) {
          (Some((b,a)),s) => {((b, c.clone(), a),s)}
          (None,s) => {((before, ccursor, after),s)}
        }
      }
    };

    before = before2;
    after = after2;
    ccursor = ccursor2;
    stat = s;
  }

  ((before, ccursor, after), stat)
}

pub fn build_content(keys: &List<Action>, mut stat: SpecStats) -> ((List<Symbol>,List<Symbol>), SpecStats) {
  let ((commands, _),stat) = undobuff_of_actions(&keys.rev(), stat);
  let ((before, _, after), stat) = content_of_commands(&commands.rev(), stat);    
  ((before, after), stat)
}

pub fn makelines(before: &List<Symbol>, after: &List<Symbol>, mut stat: SpecStats, addbar: bool, showcursors: bool) -> (List<String>, SpecStats) {
  let mut out: List<String> = List::new();
  let mut partial: String = "".to_string();
  let (max_lines_before, max_lines_after) = (40, 20) ;
  let mut count = max_lines_before; //HACK: draw off the screen sometimes to make sure the screen is full

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

  count = max_lines_after; //cursor no lower than half screen
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

  (out, stat)
}

impl EditorPipeline for SpecEditor {
  fn take_action(self: &mut Self, ac: Action, log: Option<&mut File>) -> () {
    println!("take_action {}: {:?}", self.next_id, ac);
    self.next_id += 1 ;
    self.total_actions += 1 ;
    self.last_action = Some(ac.clone());

    self.actions = self.actions.append(ac);
  }

  fn get_lines(self: &mut Self, vp: &ViewParams, log: Option<&mut File>) -> List<String> {
    let stat = SpecStats::new();
    let mut result = List::new();
    let time = measure_ns(|| {
      let ((before, after), stat) = build_content(&self.actions, stat);
      let (out, stat) = makelines(&before, &after, stat, vp.addcursor, vp.showcursors);
      self.last_stats = stat;
      result = out;
    });
    self.last_stats.gen_time = time;
    result
  }

  fn csv_title_line(self: &Self) -> String { "editor,action count,last action,milliseconds".to_string() }

  fn stats(self: &mut Self) -> (&CommonStats, String) {
    match self.last_action {
      None => (&self.last_stats, format!("Spec,{},None,{}", self.total_actions, self.last_stats.gen_time)),
      Some(ref a) => (&self.last_stats, format!("Spec,{},{},{}", self.total_actions, a, self.last_stats.gen_time)),
    }
    
  }

}


