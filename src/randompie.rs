use rand::{Rng};
use editor_defs::Cmdtype;

pub struct RandomPie {
    ins: u32,
    ovr: u32,
    rem: u32,
    mov: u32,
    goto: u32,
    make: u32,
    swch: u32,
    jump: u32,
    join: u32,
    undo: u32,
    undo_c: u32,
}

impl RandomPie {
  pub fn new(from_vals: Vec<u32>) -> Self {
    let ins = from_vals[0];
    let ovr = from_vals[1];
    let rem = from_vals[2];
    let mov = from_vals[3];
    let goto = from_vals[4];
    let make = from_vals[5];
    let swch = from_vals[6];
    let jump = from_vals[7];
    let join = from_vals[8];
    let undo = from_vals[9];
    let undo_c = from_vals[10];
    RandomPie {
      ins: ins,
      ovr: ovr,
      rem: rem,
      mov: mov,
      goto: goto,
      make: make,
      swch: swch,
      jump: jump,
      join: join,
      undo: undo,  
      undo_c: undo_c,  
    }
  }
  pub fn no_cursors(&self) -> Self {
    RandomPie{
      ins: self.ins,
      ovr: self.ovr,
      rem: self.rem,
      mov: self.mov,
      goto: self.goto,
      make: 0,
      swch: 0,
      jump: 0,
      join: 0,
      undo: self.undo,
      undo_c: self.undo_c,
    }
  }
  pub fn total(&self) -> u32 {
    self.ins + self.ovr + self.rem + self.mov + self.goto + self.make + self.swch + self.jump + self.join + self.undo
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
    if val < self.goto {return Cmdtype::Goto};
    let val = val - self.goto;
    if val < self.make {return Cmdtype::Make};
    let val = val - self.make;
    if val < self.swch {return Cmdtype::Swch};
    let val = val - self.swch;
    if val < self.jump {return Cmdtype::Jump};
    let val = val - self.jump;
    if val < self.join {return Cmdtype::Join};
    return Cmdtype::Undo;
  }
  pub fn undo_count(&self) -> u32 {
    self.undo_c
  }
}

