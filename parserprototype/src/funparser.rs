use std::io;
use std::collections::VecDeque;
use std::str::FromStr;
use std::rc::Rc;
mod funlist;

use funlist::*;

fn doit(c:char, input_stack:List<char>, postfix_expr:&mut Vec<String>) -> List<char> {
  let mut stack = input_stack;
  match stack.clone() {
    List::Nil => {
      //stack = List::Cons(c, stack);
      stack = push(stack, c);
      //continue;
      return stack
    },
    List::Cons(cons) => {
      let mut s_top = cons.hd.clone();
      if s_top=='*' || s_top=='/' {
        postfix_expr.push(s_top.to_string());
      }
      else {
        stack = push(stack, s_top);
      }
      while !is_empty(stack.clone()) {
        match stack.clone() {
          List::Nil => (),
          List::Cons(cons) => {
            let s_top = cons.hd.clone();
            if s_top=='*' || s_top=='/' {
              postfix_expr.push(s_top.to_string());
            }
            else {
              push(stack, s_top);
              break;
            }
          }
        }
      }
    }
  };
  push(stack, c)
}

fn infix_to_postfix(expr: String) -> Vec<String> {

    // Maintain a stack of operators and postfix string of operands
  //let mut stack: Vec<char> = Vec::new();
    let mut stack: List<char> = List::Nil;
    let mut postfix_expr: Vec<String> = Vec::new();
    let mut operand_stack: VecDeque<char> = VecDeque::new();

    for c in expr.chars() {

	if c>='0' && c<='9' {
	    // Append terminal to postfix string
	    operand_stack.push_back(c);
        }

	else if c=='*' {
            let mut operand_str = "".to_string();
            while !operand_stack.is_empty() {
                operand_str.push(operand_stack.pop_front().unwrap())
            }
            postfix_expr.push(operand_str);

	    // Pop stack until an operator of lower precedence is found, append the popped operators to the postfix string
	  // This is done to give priority for division over multiplication
          stack = doit(c, stack, &mut postfix_expr);
	}

	else if c=='/' {
            let mut operand_str = "".to_string();
            while !operand_stack.is_empty() {
                operand_str.push(operand_stack.pop_front().unwrap())
            }
            postfix_expr.push(operand_str);
	    push(stack, c);
        }

	else if c=='+' || c=='-' {

	    let mut operand_str = "".to_string();
            while !operand_stack.is_empty() {
                operand_str.push(operand_stack.pop_front().unwrap())
            }
            postfix_expr.push(operand_str);

            // Pop stack until an operator of equal precedence is found, append the popped operators to the postfix string
          // This is done to give priority for multiplication and division
          stack = doit(c, stack, &mut postfix_expr);
	}
    }
    let mut operand_str = "".to_string();
    while !operand_stack.is_empty() {
        operand_str.push(operand_stack.pop_front().unwrap())
    }
    postfix_expr.push(operand_str);

    //Append all the remaining operators to the postfix string
    
    loop {
      match stack {
        List::Nil => break,
        List::Cons(cons) => {
          postfix_expr.push(cons.hd.to_string());
          stack = *cons.tl;
        }
    }}
    //return postfix_expr.iter().cloned().collect::<String>();
    return postfix_expr;
}

fn postfix_parse(expr: Vec<String>) -> i32{
    //Maintain a stack of terminals
    let mut expr_list = List::Nil;
    
   
    for c in expr.iter() {
        //Push terminals onto stack after type casting them to i32
	if c.chars().nth(0).unwrap() >='0' && c.chars().nth(0).unwrap() <='9' {
	        expr_list = push(expr_list, i32::from_str(c).unwrap());
	//      push_char_onto_stack(expr_stack,8);  
        }
	else if c=="+" || c=="*" || c=="-" || c=="/" {

	let (d, list) = pop(expr_list);
	expr_list = Rc::try_unwrap(list).unwrap();
	let e1 = d.unwrap();

	let (d, list) = pop(expr_list);
	expr_list = Rc::try_unwrap(list).unwrap();
	let e2 = d.unwrap();

	let v: i32 = match c.as_ref(){
		"+" => e1 + e2,
		"-" => e1 - e2,
		"*" => e1 * e2,
		"/" => e2/e1,
		_ => panic!(""),
	};
	
	expr_list = push(expr_list, v);
        }
    } 

    let (d, _) = pop(expr_list);
    return d.unwrap();
}

fn main() {
    println!("Enter expression in infix format");
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok().expect("Failed to parse input");
    let postfix_expr = infix_to_postfix(input);
    println!("{}", postfix_parse(postfix_expr));
}