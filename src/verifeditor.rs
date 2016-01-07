use functional::List;
use spec::SpecEditor;
use fast::AdaptEditor;
use editor_defs::*;
use adapton::adapton_sigs::Adapton;
use adapton::collection_traits::ListT;
use adapton::collection_traits::TreeT;

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
        assert!( &lines_spec == &lines_fast ) ;
        lines_fast        
    }
}
