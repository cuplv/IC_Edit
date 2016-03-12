use functional::List;
use spec::SpecEditor;
use fast::AdaptEditor;
use editor_defs::*;
use adapton::adapton_sigs::Adapton;
use adapton::collection_traits::ListT;
use adapton::collection_traits::TreeT;
use time::Duration;
use std::fs::File;

pub struct VerifEditor<A:Adapton,L:ListT<A,Action>> {
  spec: SpecEditor,
  fast: AdaptEditor<A,L>,
}

impl<A:Adapton,L:ListT<A,Action>> VerifEditor<A,L> {
  pub fn new (mut st: A, acts:List<Action>, sparse: usize) -> VerifEditor<A,L> {
    VerifEditor{
      spec: SpecEditor::new(acts.clone()),
      fast: AdaptEditor::new(st, acts, sparse),
    }
  }
}

#[derive(Debug)]
pub struct VeriStats {
  gen_time: u64,
}
impl VeriStats {
  pub fn new() -> VeriStats {
  VeriStats{
    gen_time: 0,
  }
  }
}
impl CommonStats for VeriStats {
  fn time(self: &Self) -> u64 {
  self.gen_time
  }
}

impl<A:Adapton,L:ListT<A,Action>>
  EditorPipeline for VerifEditor<A,L>
{
  fn take_action(self: &mut Self, ac: Action, log: Option<&mut File>) -> () {
    match log {
      None => {
        self.spec.take_action(ac.clone(), None);
        self.fast.take_action(ac, None)
      }
      Some(log) => {
        self.spec.take_action(ac.clone(), Some(log));
        self.fast.take_action(ac, Some(log))
      }
    }
  }

  fn get_lines(self: &mut Self, vp: &ViewParams, log: Option<&mut File>) -> List<String> {
    let (lines_spec, lines_fast) = match log {
      None => {(
        self.spec.get_lines(vp.clone(), None),
        self.fast.get_lines(vp, None)
      )}
      Some(log) => {(
        self.spec.get_lines(vp.clone(), Some(log)),
        self.fast.get_lines(vp, Some(log))
      )}
    };
    if !( &lines_spec == &lines_fast ) {
      panic!("Not equal!\nspec: {:?}\nfast: {:?}", lines_spec, lines_fast )
    }
    lines_fast        
  }

  fn csv_title_line(self: &Self) -> String {
    self.fast.csv_title_line()
  }

  fn stats(self: &mut Self) -> (&CommonStats, String) {
    self.fast.stats()
  }
}
