use editor_defs::*;
use std::fmt::{Debug};
use std::hash::{Hash};

use functional;
use adapton::adapton_sigs::Adapton;
use adapton::collection_traits::ListT;
use adapton::collection_traits::TreeT;
use adapton::collection_traits::Dir2;
use adapton::collection_edit::ListEdit;
use adapton::collection_edit::ListZipper;
use adapton::collection_algo::tree_of_list;
use adapton::collection::List;
//use adapton::collection::List;
use adapton::collection;
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

pub fn pass_cursors
    <A:Adapton
    ,Symz:ListEdit<A,Symbol>
    >
    (st: &mut A, z:Symz::State, dir:Dir2) -> Symz::State
{        
    let (z, obs) = Symz::observe(st, z, dir.clone()) ;
    match obs {
        None => z,
        Some(Symbol::Data(_)) => { z },                
        Some(Symbol::Cur(_)) => {                
            let (z, success) = Symz::goto(st, z, dir.clone()) ;
            if success { return pass_cursors::<A,Symz>(st, z, dir) }
            else { z }
        },
    }        
}
    
pub fn content_of_cmdz
    <A:Adapton
    ,Cmds:TreeT<A,Command>
    ,Symz:ListEdit<A,Symbol>
    ,Syms:TreeT<A,Symbol>
    >
    (st: &mut A, cmds:Cmds::Tree) -> (Symz::State, Option<A::Name>, Cursor) {
        let emp = Symz::empty(st);
        Cmds::fold_lr(
            st, cmds, (emp, None, "0".to_string()),
            /* Leaf */ &|st, cmd, (z, nm, active)| {               
                let tz = {
                    let z = match cmd.clone() {
                        Command::Switch(_) => Symz::insert(st, z.clone(), Dir2::Left, Symbol::Cur(active.clone())),
                        _ => z.clone()
                    };
                    Symz::get_tree::<Syms>(st, z, Dir2::Left)
                } ;                
                let z = match cmd.clone() {
                    Command::Ins(_, dir) |
                    Command::Rem(dir) |
                    Command::Move(dir) |  
                    Command::Ovr(_, dir) => pass_cursors::<A,Symz>(st, z, dir2_of_dir(&dir)),
                    _ => z
                } ;
                let (z, active) = match cmd {
                    Command::Ins(data, dir) => { let z = Symz::insert(st, z, dir2_of_dir(&dir.opp()), Symbol::Data(data)) ; (z, active) }
                    Command::Rem(dir)       => { let (z, _) = Symz::remove(st, z, dir2_of_dir(&dir)) ; (z, active) },
                    Command::Move(dir)      => { let (z, _) = Symz::goto(st, z, dir2_of_dir(&dir)) ; (z, active) },
                    Command::Ovr(data, dir) => {
                        let (z, _, _) = Symz::replace(st, z, dir2_of_dir(&dir), Symbol::Data(data)) ;
                        let (z, _) = Symz::goto(st, z, dir2_of_dir(&dir)) ;
                        (z, active)
                    },
                    Command::Mk(cursor)     => { let z = Symz::insert(st, z, Dir2::Left, Symbol::Cur(cursor)) ; (z, active) },
                    Command::Join(cursor)   => panic!(""),

                    Command::Switch(cursor) => panic!(""),
                    Command::Jmp(cursor)    => panic!(""),
                } ;
                (z, nm, active)
            },
            /* Bin  */ &|st, _, r| r,
            /* Name */ &|st, nm2, _, (z,nm1,active)| match nm1 {
                None => (z, Some(nm2), active),
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

fn make_lines<A:Adapton>(st: &mut A, vp: &ViewParams,
                         symz:ListZipper<A,Symbol,List<A,Symbol>>) -> functional::List<String> {
    let (max_lines_before, max_lines_after) = (40, 20) ;
    let mut out_lines = functional::List::new() ;
    let mut cur_line  = "".to_string() ;

    let mut z = symz.clone() ;
    
    let mut line_count = 0;
    loop {
        let (z_, x) = ListZipper::remove(st, z, Dir2::Right) ;
        z = z_ ;
        match x {
            None => { break },
            Some(Symbol::Cur(ref c)) => {
                if vp.showcursors {cur_line = cur_line + "<" + &c + ">"}
            }
            Some(Symbol::Data(ref d)) => {
                if d == "\n" {
                    out_lines = out_lines.append(cur_line);
                    cur_line = "".to_string();
                    line_count += 1;
                    if line_count == max_lines_before { break }
                } else {cur_line = cur_line + &d}
            }
        }
    }
    line_count = 0 ;
    out_lines = out_lines.append(cur_line).rev() ;
    
    { /* Set up cur_line for next loop; add cursor, if applicable. */
        let cur = if vp.addcursor {"|"} else {""};
        match out_lines.head() {
            None    => {cur_line = cur.to_string()    ;}
            Some(t) => {cur_line = cur.to_string() + t;}
        }
        out_lines = out_lines.tail();
    }

    loop {
        let (z_, x) = ListZipper::remove(st, z, Dir2::Left) ;
        z = z_ ;
        match x {
            None => { break },
            Some(Symbol::Cur(ref c)) => {
                if vp.showcursors {cur_line = cur_line + "<" + &c + ">"}
            }
            Some(Symbol::Data(ref d)) => {
                if d == "\n" {
                    out_lines = out_lines.append(cur_line);
                    cur_line = "".to_string();
                    line_count += 1;
                    if line_count == max_lines_after { break }
                } else {cur_line = d.clone() + &cur_line}
            }
        }
    }

    out_lines = out_lines.append(cur_line) ;

    out_lines
}

impl<A:Adapton,L:ListT<A,Action>> EditorPipeline for AdaptEditor<A,L> {
    fn take_action(self: &mut Self, ac: Action) -> () {
        // XXX: Kyle and I don't know how to do this without cloning!
        println!("take_action: {:?}", ac);
        self.rev_actions =
            L::cons(&mut self.adapton_st, ac, self.rev_actions.clone())
    }
    
//   fn get_lines(self: &mut Self, vp: &ViewParams) -> functional::List<functional::List<Color,String>> {
    fn get_lines(self: &mut Self, vp: &ViewParams) -> functional::List<String> {
        println!("-----");
        
        let st = &mut self.adapton_st ;
       
        let acts = self.rev_actions.clone() ;
        let actions = tree_of_list::<A,Action,collection::Tree<A,Action,u32>,L>(st, Dir2::Right, acts) ;

        println!("actions: {:?}", actions);

        let (cmdz, _) = cmdz_of_actions::<A,collection::Tree<A,Action,u32>,ListZipper<A,Command,List<A,Command>>> (st, actions) ;
        let cmdz = ListZipper::clear_side(st, cmdz, Dir2::Right) ;
        let cmdt = ListZipper::get_tree::<collection::Tree<A,Command,u32>>(st, cmdz, Dir2::Left) ;

        println!("cmdt: {:?}", cmdt);       
        
        let (content, _, _) = content_of_cmdz::<
            A,collection::Tree<A,Command,u32>
            ,ListZipper<A,Symbol,List<A,Symbol>>
            ,collection::Tree<A,Symbol,u32>>(st, cmdt) ;

        println!("content: {:?}", content);

        if false {
            let (_, l) = ListZipper::observe(st, content.clone(), Dir2::Left) ;
            let (_, r) = ListZipper::observe(st, content, Dir2::Right) ;
            let l = match l {Some(Symbol::Data(s))=>s, _=>"".to_string()} ;
            let r = match r {Some(Symbol::Data(s))=>s, _=>"".to_string()} ;
            let out = functional::List::new() ;
            let out = out.append(r) ;
            let out = out.append("_".to_string()) ;
            let out = out.append(l) ;
            out
        }
        else {
            make_lines(st, vp, content)
        }
    }
}


