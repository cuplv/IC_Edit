//http://cglab.ca/~abeinges/blah/too-many-lists/book/

#![allow(dead_code)]
use std::rc::Rc;

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum List<T> {
  Nil,
  Cons(Cons<T>)
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Cons<T> {
  hd: T,
  tl: Rc<List<T>>,
}

pub struct Iter<'a, T:'a> {
  next: Option<&'a Cons<T>>,
}


pub fn push<T>(list: List<T>, item: T) -> List<T> {
  List::Cons(Cons{hd: item, tl:Rc::new(list)})
}

pub fn fold<'a, S, T, F>(list: &'a List<S>, accum: T, f: F) -> T
  where F:Fn(T, &'a S) -> T {
    match *list {
      List::Nil => accum,
      List::Cons(ref cons) => {
        fold(&*cons.tl, f(accum, &cons.hd), f)
      }
    }
}


pub fn pop<T>(list: List<T>) -> (Option<T>, Rc<List<T>>) {
  match list {
    List::Nil => (None, Rc::new(List::Nil)),
    List::Cons(cons) => (Some(cons.hd), cons.tl),
  }
}

pub fn filter<'a, S: Clone, F>(list: &'a List<S>, test: F) -> List<S> 
  where F:Fn(&'a S) -> bool {
    fold(list, List::Nil, |filtered, p| {
      if test(p) { push(filtered, p.clone()) }
      else { filtered }
    })
}


pub fn length<'a, S>(list: &'a List<S>) -> u32 {
  fold(list, 0, |n, _| { n + 1 })
}


pub fn compare<'a, S: PartialEq>(a: &'a List<S>, b: &'a List<S>) -> bool {
  // Returns true if both lists have equivalent elements, false if not
  if length(&a) == length(&b) {
    let lhs = fold(&a, true, |a_last, p| {
      a_last && fold(&b, false, |b_last, q| {
        b_last || (p == q)
      })
    });

    let rhs = fold(&b, true, |b_last, p| {
      b_last && fold(&a, false, |a_last, q| {
        a_last || (p == q)
      })
    });

    lhs && rhs
  } else {
    false
  }
}

#[test]
pub fn test_sum() {
  let l = List::Nil;
  let l = List::Cons(Cons{hd:3, tl:Rc::new(l)});
  let l = List::Cons(Cons{hd:2, tl:Rc::new(l)});
  let l = List::Cons(Cons{hd:1, tl:Rc::new(l)});
  let sum = fold(&l, 0, |sum,elm|{ sum + elm });
  assert_eq!(sum, 6);    
}

#[test]
pub fn test_push() {
  let l = List::Nil;
  let l = push(l, 3);
  let l = push(l, 2);
  let l = push(l, 1);
  let sum = fold(&l, 0, |sum, elm|{ sum + elm });
  assert_eq!(sum, 6);
}

#[test]
pub fn test_filter() {
  let l = List::Nil;
  let l = push(l, 3);
  let l = push(l, 2);
  let l = push(l, 1);
  let geq2 = filter(&l, |&n|{ n >= 2 });

  let sum = fold(&geq2, 0, |sum, elm|{ sum + elm });

  assert_eq!(sum, 5);
}

#[test]
pub fn test_length() {
  let l = List::Nil;
  let l = push(l, 3);
  let l = push(l, 2);
  let l = push(l, 1);
  
  let len = length(&l);

  assert_eq!(len, 3);  
}

#[test]
pub fn test_compare() {
  let l = List::Nil;
  let l = push(l, 4);
  let l = push(l, 3);
  let l = push(l, 2);
  let l = push(l, 1);

  let m = List::Nil;
  let m = push(m, 1);
  let m = push(m, 2);
  let m = push(m, 3);
  let m = push(m, 4);

  assert!(compare(&l, &m));
}
