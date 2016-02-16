use rand::{Rng};
use editor_defs::Cmdtype;

pub struct RandomPie {
    ins: u32,
    ovr: u32,
    rem: u32,
    mov: u32,
    make: u32,
    swch: u32,
    jump: u32,
    join: u32,
    redo: u32,
    undo: u32,
}

impl RandomPie {
  pub fn new(from_vals: Vec<u32>) -> Self {
    let ins = from_vals[0];
    let ovr = from_vals[1];
    let rem = from_vals[2];
    let mov = from_vals[3];
    let make = from_vals[4];
    let swch = from_vals[5];
    let jump = from_vals[6];
    let join = from_vals[7];
    let redo = from_vals[8];
    let undo = from_vals[9];
    RandomPie {
      ins: ins,
      ovr: ovr,
      rem: rem,
      mov: mov,
      make: make,
      swch: swch,
      jump: jump,
      join: join,
      redo: redo,
      undo: undo,  
    }
  }
  pub fn no_cursors(&self) -> Self {
    RandomPie{
      ins: self.ins,
      ovr: self.ovr,
      rem: self.rem,
      mov: self.mov,
      make: 0,
      swch: 0,
      jump: 0,
      join: 0,
      redo: self.redo,
      undo: self.undo,
    }
  }
  pub fn total(&self) -> u32 {
    self.ins + self.ovr + self.rem + self.mov + self.make + self.swch + self.jump + self.join + self.redo + self.undo
  }
  pub fn get_cmd_type<R: Rng>(&self, rng: &mut R) -> Cmdtype {
    let val = rng.gen_range(0, self.total());
    if val < self.ins {return Cmdtype::Ins};
    let val = val - self.ins;
    if val < self.ovr {return Cmdtype::Ovr};
    let val = val - self.ovr;
    if val < self.rem {return Cmdtype::Rem};
    let val = val - self.rem;
    if val < self.mov {return Cmdtype::Mov};
    let val = val - self.mov;
    if val < self.make {return Cmdtype::Make};
    let val = val - self.make;
    if val < self.swch {return Cmdtype::Swch};
    let val = val - self.swch;
    if val < self.jump {return Cmdtype::Jump};
    let val = val - self.jump;
    if val < self.join {return Cmdtype::Join};
    let val = val - self.join;
    if val < self.redo {return Cmdtype::Redo};
    return Cmdtype::Undo;
  }
}

