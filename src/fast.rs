use editor_defs::*;

use functional;
use adapton::adapton_sigs::Adapton;
use adapton::collection_traits::ListT;
use adapton::collection::List;

pub struct AdaptEditor<A:Adapton> {
    actions: List<A,Action>
}

pub fn list_of_list<A:Adapton,L:ListT<A,Action>>
    (st: &mut A, l_in:functional::List<Action>) -> L::List
{
    let mut l_out = L::nil(st);
    for x in l_in.rev().iter() { l_out = L::cons(st,x.clone(),l_out) }
    return l_out
}

impl<A:Adapton> AdaptEditor<A> {
  pub fn new(st: &mut A, initial_actions: functional::List<Action>) -> AdaptEditor<A> {
    AdaptEditor{
      actions: list_of_list::<A,List<A,Action>>(st, initial_actions)
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


