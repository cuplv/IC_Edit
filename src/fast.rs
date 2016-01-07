use editor_defs::*;
use std::fmt::{Debug};
use std::hash::{Hash};

use functional;
use adapton::adapton_sigs::Adapton;
use adapton::collection_traits::ListT;
use adapton::collection_traits::TreeT;
use adapton::collection_traits::Dir2;
use adapton::collection_edit::ListEdit;
//use adapton::collection::List;
use std::ops::Add;
use std::num::Zero;

pub struct AdaptEditor<A:Adapton,L:ListT<A,Action>> {
    adapton_st: A,
    rev_actions: L::List,
}

pub fn list_of_list<A:Adapton,L:ListT<A,Action>>
    (st: &mut A, l_in:functional::List<Action>) -> L::List
{
    let mut l_out = L::nil(st);
    for x in l_in.rev().iter() { l_out = L::cons(st,x.clone(),l_out) }
    return l_out
}

impl<A:Adapton,L:ListT<A,Action>> AdaptEditor<A,L> {
    pub fn new(mut st: A, initial_actions: functional::List<Action>) -> AdaptEditor<A,L> {
        let actions = list_of_list::<A,L>(&mut st, initial_actions) ;
      AdaptEditor{
        adapton_st: st,
        rev_actions: actions,
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
                Symbol::Cur(cursor)   => ContentInfo{ cursors:vec![cursor], data_count:0, line_count:0 },
                Symbol::Data(ref string)
                    if string == "\n" => ContentInfo{ cursors:vec![],       data_count:0, line_count:1 },
                Symbol::Data(string)  => ContentInfo{ cursors:vec![],       data_count:1, line_count:0 },
            }
        },
        &|st,  _,l,r| (l + r),
        &|st,_,_,l,r| (l + r),
        )
}

pub fn dir2_of_dir (d:&Dir) -> Dir2 {
    match *d {
        Dir::L => Dir2::Left,
        Dir::R => Dir2::Right,
    }
}

pub fn content_of_cmdz
    <A:Adapton
    ,Cmds:TreeT<A,Command>
    ,Symz:ListEdit<A,Symbol>
    >
    (st: &mut A, cmds:Cmds::Tree) -> (Symz::State, Option<A::Name>) {
        let emp = Symz::empty(st);
        Cmds::fold_lr(
            st, cmds, (emp, None),
            /* Leaf */ &|st, cmd, (z, nm)| {
                let z = match cmd {
                    Command::Ins(data, dir) => Symz::insert(st, z, dir2_of_dir(&dir), Symbol::Data(data)),
                    Command::Rem(dir)       => { let (z, _) = Symz::remove(st, z, dir2_of_dir(&dir)) ; z },
                    Command::Move(dir)      => { let (z, _) = Symz::goto(st, z, dir2_of_dir(&dir)) ; z },
                    Command::Ovr(data, dir) => {
                        let (z, _, _) = Symz::replace(st, z, dir2_of_dir(&dir), Symbol::Data(data)) ;
                        let (z, _) = Symz::goto(st, z, dir2_of_dir(&dir)) ;
                        z
                    },
                    _ => panic!("")
                } ;
                (z, nm)
            },
            /* Bin  */ &|st, _, r| r,
            /* Name */ &|st, nm2, _, (z,nm1)| match nm1 {
                None => (z, Some(nm2)),
                Some(_) => panic!("nominal ambiguity!")
            },
            )
        }

pub fn cmdz_of_actions
    <A:Adapton
    ,Acts:TreeT<A,Action>
    ,Edit:ListEdit<A,Command>
    >
    (st: &mut A, acts:Acts::Tree)
     -> (Edit::State, Option<A::Name>) {
         let emp = Edit::empty(st);
         Acts::fold_lr(
             st, acts, (emp, None),
             /* Leaf */ &|st, act, (z,nm)| {
                 match act {
                     Action::Undo => {
                         let (z,_) = Edit::goto(st, z, Dir2::Left);
                         (z,nm)
                     },                         
                     Action::Redo => {
                         let (z,_) = Edit::goto(st, z, Dir2::Right);
                         (z,nm)
                     },
                     Action::Cmd(c) => {
                         let z = match nm {
                             None => Edit::insert(st, z, Dir2::Left, c),
                             Some(nm) => {
                                 // TODO: Use nm
                                 Edit::insert(st, z, Dir2::Left, c)
                             }} ;
                         (z, None)
                     }
                 }},
             /* Bin  */ &|st, _, r| r,
             /* Name */ &|st, nm2, _, (z,nm1)| match nm1 {
                 None => (z, Some(nm2)),
                 Some(_) => panic!("nominal ambiguity!")
             },
             )
     }

impl<A:Adapton,L:ListT<A,Action>> EditorPipeline for AdaptEditor<A,L> {
    fn take_action(self: &mut Self, ac: Action) -> () {
        // XXX: Kyle and I don't know how to do this without cloning!
        self.rev_actions =
            L::cons(&mut self.adapton_st, ac, self.rev_actions.clone())
    }

    fn get_lines(self: &Self, vp: &ViewParams) -> functional::List<String> {
        panic!("")
    }
}


