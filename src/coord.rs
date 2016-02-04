use std::fs::File;
use std::fmt::Debug;
use std::hash::Hash;

use time;
use gm;
use adapton::collection::Tree;
use adapton::adapton_sigs::Adapton;

// #[derive(Debug,PartialEq,Eq,Hash,Clone)]
// pub enum Tree<A:Adapton,X,Lev> {
//     Nil,
//     Leaf(X),
//     Bin(          Lev, Box<Tree<A,X,Lev>>, Box<Tree<A,X,Lev>> ),
//     Name(A::Name, Lev, Box<Tree<A,X,Lev>>, Box<Tree<A,X,Lev>> ),
//     Rc(                 Rc<Tree<A,X,Lev>>),
//     Art(               Art<Tree<A,X,Lev>, A::Loc>),
// }
 

fn rec_desc<A:Adapton,E:Debug+Hash+PartialEq+Eq+Clone,L:Hash+Debug+Eq+Clone>
(st: &mut A, t: &Tree<A,E,L>, f: &mut File, up: &str, side: &str) {
  let leaf = "green";
  let bin = "yellow";
  let name = "orange";
  let rc = "yellow";
  let art = "blue";
  fn edge(f: &mut File, from: &str, to: &str) {
      gm::addedge(f, from, to, "", "black", "", None)
  }
  match *t {
    Tree::Nil => {}
    Tree::Leaf(ref d) => {
      let node = format!("{:?}",d);
      gm::addnode(f, &node, leaf, "", None);
      edge(f, up, &node);
    }
    Tree::Bin(_, ref t1, ref t2) => {
      let node = format!("{}b{}",up,side);
      gm::addnode(f, &node, bin, "", None);
      edge(f, up, &node);
      rec_desc(st, &**t1, f, &node, "l");
      rec_desc(st, &**t2, f, &node, "r");
    }
    Tree::Name(ref n,_, ref t1, ref t2) => {
      let node = format!("{:?}", n);
      gm::addnode(f, &node, name, "", None);
      edge(f, up, &node);
      rec_desc(st, &**t1, f, &node, "l");
      rec_desc(st, &**t2, f, &node, "r");          
    }
    Tree::Rc(ref t) => {
      let node = format!("{}r{}",up,side);
      gm::addnode(f, &node, rc, "", None);
      edge(f, up, &node);
      rec_desc(st, &**t, f, &node, "c");
    }
    Tree::Art(ref a) => {
      let node = format!("{:?}", a);
      let t = st.force(a);
      gm::addnode(f, &node, art, "", None);
      edge(f, up, &node);
      rec_desc(st, &t, f, &node, "c");          
    }
  }
}

impl<A:Adapton,E:Debug+Hash+PartialEq+Eq+Clone,L:Hash+Debug+Eq+Clone>
gm::GMLog<A> for Tree<A,E,L> {
  fn log_snapshot(self: &Self, st: &mut A, file: &mut File, msg: Option<&str>) {
    gm::startframe(file, &format!("Logged at {}", time::now().asctime()), msg);
    rec_desc(st, self, file, "root", "c");
  }
}
