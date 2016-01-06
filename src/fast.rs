use editor_defs::*;

use functional;
use adapton::adapton_sigs::Adapton;
use adapton::collection::List;

pub struct AdaptEditor<A:Adapton> {
    actions: List<A,Action>
}

pub fn list_of_list<A:Adapton> (l:functional::List<Action>) -> List<A,Action> {
    panic!("")
}

impl<A:Adapton> AdaptEditor<A> {
  pub fn new(initial_actions: functional::List<Action>) -> AdaptEditor<A> {
    AdaptEditor{
      actions: list_of_list(initial_actions)
    }
  }
}

// pub fn tree_reduce_monoid<A:Adapton,Elm:Eq+Hash+Clone+Debug,T:TreeT<A,Elm>,BinOp>
//     (st:&mut A, tree:T::Tree, zero:Elm, binop:&BinOp) -> Elm
//     where BinOp:Fn(&mut A, Elm, Elm) -> Elm
// {
//     T::fold_up(st, tree,
//                         &|_| zero.clone(),
//                    &|_,leaf| leaf,
//                  &|st,_,l,r| binop(st,l,r),
//                &|st,_,_,l,r| binop(st,l,r),
//                )
// }






impl<A:Adapton> EditorPipeline for AdaptEditor<A> {
    fn take_action(self: &mut Self, ac: Action) -> () {
        panic!("")
    }

    fn get_lines(self: &Self, vp: &ViewParams) -> functional::List<String> {
        panic!("")
    }
}


