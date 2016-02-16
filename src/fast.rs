use std::fmt::{Debug};
use std::hash::{Hash};
use std::rc::Rc;
use std::ops::Add;
use std::num::Zero;
use time::Duration;
use std::fs::File;

use editor_defs::*;
use functional;
use adapton::gm::GMLog;
use adapton::adapton_sigs::*;
use adapton::collection_traits::ListT;
use adapton::collection_traits::TreeT;
use adapton::collection_traits::Dir2;
use adapton::collection_edit::ListEdit;
use adapton::collection_edit::ListZipper;
use adapton::collection_algo::tree_of_list;
use adapton::collection::List;
use adapton::collection::Tree;
use adapton::collection;
use adapton::macros::* ;

#[derive(Debug)]
pub struct AdaptonStats {
  gen_time: Duration,
}
impl AdaptonStats {
  pub fn new() -> AdaptonStats {
    AdaptonStats{
      gen_time: Duration::zero(),
    }
  }
}
impl CommonStats for AdaptonStats {
  fn time(self: &Self) -> Duration {
    self.gen_time
  }
}

pub struct AdaptEditor<A:Adapton,L:ListT<A,Action>> {
  next_id: usize,
  adapton_st: A,
  last_stats: AdaptonStats,
  last_action: Option<Action>,
  rev_actions: L::List,
}

impl<A:Adapton,L:ListT<A,Action>> AdaptEditor<A,L> {
  pub fn new(mut st: A, initial_actions: functional::List<Action>) -> AdaptEditor<A,L> {

    let mut more_acs = initial_actions ;
    let mut actions = L::nil(&mut st) ;
    let mut id = 0 ;

    loop {
      let ac = if let Some(a) = more_acs.head() {a.clone()} else {break} ;
      let nm = st.name_of_usize(id) ;
      let (nm1,nm2) = st.name_fork(nm) ;
      actions = {
        let l   = L::cons(&mut st, ac, actions) ;
        let art = st.cell( nm1, l ) ;
        let art = st.read_only( art ) ;
        let l   = L::art(&mut st, art) ;
        let l   = L::name(&mut st, nm2, l) ;
        l
      } ;
      id += 1 ;
      more_acs = more_acs.tail() ;
    }

    AdaptEditor{
      next_id: id,
      adapton_st: st,
      last_stats: AdaptonStats::new(),
      last_action: None,
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

impl<A:Adapton> GMLog<A> for ContentInfo {}

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

pub fn tree_focus<A:Adapton,T:TreeT<A,Symbol>,Symz:ListEdit<A,Symbol,T>>
  (st:&mut A, tree:T::Tree, cur:Cursor, symz:Symz::State) -> Option<Symz::State> {
    T::elim_move
      (st, tree, (cur, symz),
       /* Empty */ |st, (cur, symz)| None,
       /* Leaf */  |st, sym, (cur, symz)| match sym {
         Symbol::Cur(ref c) if c == &cur => { Some(symz) },
         _ => None,
       },
       /* Bin */ |st, _, l, r, (cur, symz)| {
         let li = tree_info::<A,T>(st, l.clone()) ;
         let ri = tree_info::<A,T>(st, r.clone()) ;
         if li.cursors.contains( &cur )
         {
           let symz = Symz::ins_tree(st, symz, Dir2::Right, r, Dir2::Left);
           return tree_focus::<A,T,Symz>(st, l, cur, symz)
         }
         else if ri.cursors.contains( &cur )
         {
           let symz = Symz::ins_tree(st, symz, Dir2::Left, l, Dir2::Right);
           return tree_focus::<A,T,Symz>(st, r, cur, symz)
         }
         else
         { None }
       },
       // Todo-Sometime: Make a combinator for this (common case): The
       // Name and Bin case are identical; avoid duplicate
       // code-related errors across bug fixes.
       /* Name */ |st, _, _, l, r, (cur, symz)| {
         let li = tree_info::<A,T>(st, l.clone()) ;
         let ri = tree_info::<A,T>(st, r.clone()) ;
         if li.cursors.contains( &cur )
         {
           let symz = Symz::ins_tree(st, symz, Dir2::Right, r, Dir2::Left);
           return tree_focus::<A,T,Symz>(st, l, cur, symz)
         }
         else if ri.cursors.contains( &cur )
         {
           let symz = Symz::ins_tree(st, symz, Dir2::Left, l, Dir2::Right);
           return tree_focus::<A,T,Symz>(st, r, cur, symz)
         }
         else
         { None }
       }
       )
  }


pub fn tree_info<A:Adapton,T:TreeT<A,Symbol>>
  (st:&mut A, tree:T::Tree) -> ContentInfo
{
  st.structural(|st|{
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
    )})
}

pub fn dir2_of_dir (d:&Dir) -> Dir2 {
  match *d {
    Dir::L => Dir2::Left,
    Dir::R => Dir2::Right,
  }
}

pub fn pass_cursors
  <A:Adapton
  ,T:TreeT<A,Symbol>
  ,Symz:ListEdit<A,Symbol,T>
  >
  (st: &mut A, z:Symz::State, dir:Dir2) -> Symz::State
{
  let (_, obs) = Symz::observe(st, z.clone(), dir.clone()) ;
  match obs {
    None => z,
    Some(Symbol::Data(_)) => { z },
    Some(Symbol::Cur(_)) => {
      // Todo-Later goto operation does not insert new names
      let (z, success) = Symz::move_optnm(st, z, dir.clone(), None) ;
      if success {
        return pass_cursors::<A,T,Symz>(st, z, dir)
      }
      else { z }
    },
  }        
}

pub fn content_of_cmdz
  <A:Adapton
  ,Cmds:TreeT<A,Command>
  ,Syms:TreeT<A,Symbol>
  ,Symz:ListEdit<A,Symbol,Syms>
  >
  (st: &mut A, cmds:Cmds::Tree) -> (Symz::State, Option<A::Name>, Cursor) {
    let emp = Symz::empty(st);
    Cmds::fold_lr(
      st, cmds, (emp, None, "0".to_string()),
      /* Leaf */ &|st, cmd, (z, optnm, active) | {
        let tz = {
          let z = match cmd.clone() {
            Command::Switch(_) =>
              // XXX: Do we need names/arts?
              Symz::insert(st, z.clone(), Dir2::Left, Symbol::Cur(active.clone())),
            _ => z.clone()
          };
          // Bug fix: Need to do this get_tree operation
          // structurally, not nominally! In general, names *will* be
          // re-associated in the tree throughout the dynamic extent
          // of this Cmds::fold_lr call.
          st.structural(|st| { Symz::get_tree(st, z, Dir2::Left) })
        } ;
        //let msg = last_action.map(|ac| format!("last action: {:?}", ac));
        //let msg = msg.as_ref().map(String::as_ref); // convert Option<String> to Option<&str>
        //tz.log_snapshot(st, None);

        let info = tree_info::<A,Syms> (st, tz.clone() ) ;
        let z = match cmd.clone() {
          Command::Ins(_, dir) |
          Command::Rem(dir) |
          Command::Move(dir) |  
          Command::Ovr(_, dir) => pass_cursors::<A,Syms,Symz>(st, z, dir2_of_dir(&dir)),
          _ => z
        } ;
        let (z, optnm_next, active) : (_,Option<A::Name>,_) = match cmd {

          Command::Ins(data, dir) => {
            let z = Symz::insert_optnm(st, z, dir2_of_dir(&dir).opp(), optnm, Symbol::Data(data)) ;
            (z, None, active)
          },

          Command::Rem(dir) => {
            let (z, _) = Symz::remove(st, z, dir2_of_dir(&dir)) ;
            // XXX Return nm or None here?
            (z, None, active)
          },

          Command::Move(dir) => {
            let (z, _) = Symz::move_optnm(st, z, dir2_of_dir(&dir), optnm) ;
            (z, None, active)
          },

          Command::Ovr(data, dir) => {
            let (z, _) = Symz::remove(st, z, dir2_of_dir(&dir)) ;
            let z = Symz::insert_optnm(st, z, dir2_of_dir(&dir.opp()), optnm, Symbol::Data(data)) ;
            (z, None, active)
          },
          
          Command::Mk(cursor) => {
            let z = Symz::insert_optnm(st, z, Dir2::Left, optnm, Symbol::Cur(cursor)) ;
            (z, None, active)
          },

          Command::Join(cursor) =>
          { let z_new = Symz::empty(st);
            let z_new = tree_focus::<A,Syms,Symz>(st, tz, cursor.clone(), z_new) ;
            match z_new {
              None => (z, None, active),
              Some(z) => (z, None, cursor),
            }},

          Command::Switch( cursor) => { // XXX: This is still broken, unfortunately.
            if cursor == active { (z, None, active) } else {
            let z_new = Symz::empty(st);
            let z_new = tree_focus::<A,Syms,Symz>(st, tz, cursor.clone(), z_new) ;
            match z_new {
              None =>        (z, None, active),
              Some(new_z) => (new_z, None, cursor),
            }}},

          Command::Jmp(cursor) => {
            let z_new = Symz::empty(st);
            let z_new = tree_focus::<A,Syms,Symz>(st, tz, cursor.clone(), z_new) ;
            match z_new {
              None => (z, None, active),
              Some(z) => {
                let z = Symz::insert_optnm(st, z, Dir2::Left, optnm, Symbol::Cur(cursor));
                (z, None, active)
              }
            }},
          //_ => panic!("not handled")
        } ;
        (z, optnm_next, active)
      },
      /* Bin  */ &|st, _, r| r,

      /* Name */ &|st, nm2, _, (z,nm1,active)| match nm1 {
        None => {
          //println!("*** content_of_cmdz: None {:?}", &nm2) ;
          (z, Some(nm2), active)
        },
        Some(nm1) => {
          // XXXX FIX ME!
          //panic!("nominal ambiguity! Should we use {:?} or {:?} ?", nm1, nm2)
          //println!("*** content_of_cmdz: Some({:?}) {:?}", &nm1, &nm2) ;
          (z, Some(nm2), active)
        }
      },
      )
  }

pub fn cmdz_of_actions
  <A:Adapton
  ,Acts:TreeT<A,Action>
  ,Cmds:TreeT<A,Command>
  ,Edit:ListEdit<A,Command,Cmds>
  >
  (st: &mut A, acts:Acts::Tree)
   -> (Edit::State, Option<A::Name>) {
     let emp = Edit::empty(st);
     Acts::fold_lr(
       st, acts, (emp, None),
       /* Leaf */ &|st, act, (z,nm)| {
         match act {
           Action::Undo => {
             let (z,_) = Edit::move_optnm(st, z, Dir2::Left, nm);
             (z,None)
           },                         
           Action::Redo => {
             let (z,_) = Edit::move_optnm(st, z, Dir2::Right, nm);
             (z,None)
           },
           Action::Cmd(c) => {
             let z = Edit::insert_optnm(st, z, Dir2::Left, nm, c);
             let z = Edit::clear_side(st, z, Dir2::Right);
             (z, None)
           }
         }},
       /* Bin  */ &|st, _, r| r,
       /* Name */ &|st, nm2, _, (z,nm1)| match nm1 {
         None => (z, Some(nm2)),
         Some(_) =>
          // XXXX FIX ME!
          //panic!("nominal ambiguity! Should we use {:?} or {:?} ?", nm1, nm2)
          (z, Some(nm2))
       },
       )
   }

fn make_lines<A:Adapton>(st: &mut A, vp: &ViewParams,
                         symz:ListZipper<A,Symbol,Tree<A,Symbol,u32>,List<A,Symbol>>) -> functional::List<String> {
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
        if vp.showcursors {cur_line = "<".to_string() + &c + ">" + &cur_line}
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


// macro_rules! namespace {
//   ( $st:expr , $name:expr =>> $code:expr ) =>
//   {{
//     let nm = ($st).name_of_string( ($name).to_string() ) ;
//     ($st).ns(nm, |st| {
//       $code
//     })}};
// }

impl<A:Adapton,L:ListT<A,Action>> EditorPipeline for AdaptEditor<A,L> {
  fn take_action(self: &mut Self, ac: Action, log: Option<&mut File>) -> () {
    // XXX: Kyle and I don't know how to do this without cloning!
    // Done: Need to insert names and articulations into this list
    println!("take_action {}: {:?}", self.next_id, ac);
    self.last_action = Some(ac.clone());
    let id = self.next_id ;
    self.next_id += 1 ;
    let sparse_count = 2;
    if true { //self.next_id % sparse_count == 0 {
      let nm = self.adapton_st.name_of_usize(id) ;
      let (nm1,nm2) = self.adapton_st.name_fork(nm) ;
      self.rev_actions = {
        let l   = self.rev_actions.clone() ;
        let l   = L::cons(&mut self.adapton_st, ac, l) ;
        let art = self.adapton_st.cell( nm1, l ) ;
        let art = self.adapton_st.read_only( art ) ;
        let l   =  L::art(&mut self.adapton_st, art) ;
        let l   = L::name(&mut self.adapton_st, nm2, l) ;
        l
      }
    } else {
      self.rev_actions = L::cons(&mut self.adapton_st, ac, self.rev_actions.clone()) ;
    }
  }
  
  //   fn get_lines(self: &mut Self, vp: &ViewParams) -> functional::List<functional::List<Color,String>> {
  fn get_lines(self: &mut Self, vp: &ViewParams, log: Option<&mut File>) -> functional::List<String> {
    self.last_stats.gen_time = Duration::zero();
    let mut result = functional::List::new();
    let acts = self.rev_actions.clone() ;
    let iter_count = self.next_id.clone() ;
    let last_action = self.last_action.clone() ;
    let mut log = log; // required to pass value into closure along with .take()
    let (time, cnt) =
      self.adapton_st.cnt(|st| { 
        let time = Duration::span(|| {
          println!("----- {}", iter_count);
          
          let (actiont, actiont_cnt) = st.cnt(|st| {
            let nm = st.name_of_string("tree_of_list".to_string()) ;
            st.ns(nm, |st| {
              tree_of_list::<A,Action,collection::Tree<A,Action,u32>,L>(st, Dir2::Right, acts)                
            })}) ;

          
          println!("actiont: {:?} {:?}", actiont_cnt, actiont);

          let (cmdz, cmdz_cnt) = st.cnt(|st| {
            let (cmdz, _) = cmdz_of_actions::<A
              ,collection::Tree<A,Action,u32>
              ,collection::Tree<A,Command,u32>
              ,ListZipper<A,Command,Tree<A,Command,u32>,List<A,Command>>> (st, actiont) ;
            cmdz
          }) ;
          
          println!("cmdz:    {:?} {:?}", cmdz_cnt, cmdz);

          let cmdz = ListZipper::clear_side(st, cmdz, Dir2::Right) ;          
          let (cmdt, cmdt_cnt) = st.cnt(|st|{
            let nm = st.name_of_string("get_tree".to_string()) ;
            st.ns(nm, |st| {
              ListZipper::get_tree(st, cmdz, Dir2::Left)
            })}) ;
          
          println!("cmdt:    {:?} {:?}", cmdt_cnt, cmdt);

          let (content, content_cnt) = st.cnt(|st|{
            let nm = st.name_of_string("content_of_cmdz".to_string()) ;
            st.ns(nm.clone(), |st| {
              let dummy = 0 ; // XXX: Workaround for Rust macro issue
              let nm_wrapper = st.name_of_string("content_of_cmdz".to_string()) ;
              let (content, _, _) =
                content_of_cmdz::<A,collection::Tree<A,Command,u32>,collection::Tree<A,Symbol,u32>,ListZipper<A,Symbol,collection::Tree<A,Symbol,u32>,List<A,Symbol>>>(st, cmdt) ;
              content }) }) ;
          
          // log output
          //let msg = last_action.map(|ac| format!("last action: {:?}", ac));
          //let msg = msg.as_ref().map(String::as_ref); // convert Option<String> to Option<&str>
          //if let Some(log) = log.take() {content.log_snapshot(st, log, msg)};
          
          println!("content: {:?} {:?}", content_cnt, content);
          
          result = make_lines(st, vp, content) ;
        }) ;
        time      
      }) ;
    println!("{:?}", cnt) ;
    
    self.last_stats.gen_time = time;
    result
  }

  fn csv_title_line(self: &Self) -> String { "editor, milliseconds".to_string() }

  fn stats(self: &mut Self) -> (&CommonStats, String) {
    (&self.last_stats, format!("Fast, {}", self.last_stats.gen_time.num_milliseconds()))
  }

}
