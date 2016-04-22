
mod funlist;
use funlist::*;

enum Dig {
  D0, D1, D2,
}

fn test_dig (x:char) {
  let mut l : List<Dig> = List::Nil;  
  let d = match x {
    '0' => Dig::D0,
    '1' => Dig::D1,
    '2' => Dig::D2,
    _ => panic!(""),
  };
  push(l, d);
}

fn test_push () {
  let mut l : List<usize> = List::Nil;
  let v : Vec<usize> = vec![0,1,2,3];
  for x in v {
    l = push(l, x);
  }
  println!("{:?}", l);
}

fn digs_of_chars (lin:List<char>) -> List<Dig> {
  fold(&lin, List::Nil, |lac, c|{
    let d = match c {
      &'0' => Dig::D0,
      &'1' => Dig::D1,
      &'2' => Dig::D2,
      _ => panic!(""),
    };
    push(lac,d)
  })
}

fn starts_with_zero (lin:List<Dig>) -> bool {
  let (d,_) = pop(lin);
  match d {
    Some(Dig::D0) => true,
    _ => false,
  }    
}

fn main () {

}
