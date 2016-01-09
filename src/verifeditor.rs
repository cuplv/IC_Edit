use functional::List;
use spec::SpecEditor;
use fast::AdaptEditor;
use editor_defs::*;
use adapton::adapton_sigs::Adapton;
use adapton::collection_traits::ListT;
use adapton::collection_traits::TreeT;
use time::Duration;

pub struct VerifEditor<A:Adapton,L:ListT<A,Action>> {
    spec: SpecEditor,
    fast: AdaptEditor<A,L>,
}

impl<A:Adapton,L:ListT<A,Action>> VerifEditor<A,L> {
    pub fn new (mut st: A, acts:List<Action>) -> VerifEditor<A,L> {
        VerifEditor{
            spec: SpecEditor::new(acts.clone()),
            fast: AdaptEditor::new(st, acts),
        }
    }
}

#[derive(Debug)]
pub struct VeriStats {
  gen_time: Duration,
}
impl VeriStats {
  pub fn new() -> VeriStats {
    VeriStats{
      gen_time: Duration::zero(),
    }
  }
}
impl CommonStats for VeriStats {
  fn time(self: &Self) -> Duration {
    self.gen_time
  }
}

impl<A:Adapton,L:ListT<A,Action>>
    EditorPipeline for VerifEditor<A,L>
{
    fn take_action(self: &mut Self, ac: Action) -> () {
        self.spec.take_action(ac.clone());
        self.fast.take_action(ac)
    }

    fn get_lines(self: &mut Self, vp: &ViewParams) -> List<String> {
        let lines_spec = self.spec.get_lines(vp.clone());
        let lines_fast = self.fast.get_lines(vp);
        if !( &lines_spec == &lines_fast ) {
            panic!("Not equal!\nspec: {:?}\nfast: {:?}", lines_spec, lines_fast )
        }
        lines_fast        
    }

    fn stats(self: &mut Self) -> (&CommonStats, String) {
        self.fast.stats()
    }
}
