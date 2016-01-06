use editor_defs::*;
use std::fmt::{Debug};
use std::hash::{Hash};

use functional;
use adapton::adapton_sigs::Adapton;
use adapton::collection_traits::ListT;
use adapton::collection_traits::TreeT;
use adapton::collection::List;
use std::ops::Add;
use std::num::Zero;

pub struct AdaptEditor<A:Adapton> {
    rev_actions: List<A,Action>
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
        rev_actions: list_of_list::<A,List<A,Action>>(st, initial_actions)
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentInfo {
    cursors:Vec<Cursor>,
    data_count:usize,
    line_count:usize,
}

impl Add for ContentInfo {
    type Output=ContentInfo;
    fn add(self, rhs: Self) -> Self::Output {
        ContentInfo {
            cursors    : { let mut v = self.cursors.clone() ; v.append( &mut rhs.cursors.clone() ) ; v },
            data_count : self.data_count + rhs.data_count,
            line_count : self.line_count + rhs.line_count,
        }
    }
}

impl Zero for ContentInfo {
    fn zero() -> Self {
        ContentInfo {
            cursors    : vec![],
            data_count : 0,
            line_count : 0,
        }
    }
}



pub fn tree_info<A:Adapton,T:TreeT<A,Symbol>>
    (st:&mut A, tree:T::Tree) -> ContentInfo
{
    T::fold_up(
        st, tree,
        &|_|      ContentInfo::zero(),
        &|_,leaf| {
            match leaf {
                Symbol::Cur(cursor)  => ContentInfo{ cursors:vec![cursor], data_count:0, line_count:0 },
                Symbol::Data(ref string) if string == "\n" => ContentInfo{ cursors:vec![], data_count:0, line_count:1 },
                Symbol::Data(string) => ContentInfo{ cursors:vec![], data_count:1, line_count:0 },
            }
        },
        &|st,  _,l,r| (l + r),
        &|st,_,_,l,r| (l + r),
        )
}


impl<A:Adapton> EditorPipeline for AdaptEditor<A> {
    fn take_action(self: &mut Self, ac: Action) -> () {
        panic!("")
    }

    fn get_lines(self: &Self, vp: &ViewParams) -> functional::List<String> {
        panic!("")
    }
}


