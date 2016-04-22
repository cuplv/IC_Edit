use std::io;
use std::collections::VecDeque;
use std::str::FromStr;
use std::rc::Rc;
mod funlist;

use funlist::*;

fn infix_to_postfix(expr: String) -> Vec<String> {

    // Maintain a stack of operators and postfix string of operands
    let mut stack: Vec<char> = Vec::new();
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
            if stack.len() == 0 {
                stack.push(c);
                continue;
            }
            else {
                let mut s_top = stack.pop().unwrap();
                if s_top=='*' || s_top=='/' {
                    postfix_expr.push(s_top.to_string());
                }
                else {
                    stack.push(s_top);
                }
                while stack.len() > 0 {
                    s_top = stack.pop().unwrap();
                    if s_top=='*' || s_top=='/' {
                        postfix_expr.push(s_top.to_string());
                    }
                    else {
                        stack.push(s_top);
                        break;
                    }
                }
            }
	    stack.push(c);
	}

	else if c=='/' {
            let mut operand_str = "".to_string();
            while !operand_stack.is_empty() {
                operand_str.push(operand_stack.pop_front().unwrap())
            }
            postfix_expr.push(operand_str);
	    stack.push(c);
        }

	else if c=='+' || c=='-' {

	    let mut operand_str = "".to_string();
            while !operand_stack.is_empty() {
                operand_str.push(operand_stack.pop_front().unwrap())
            }
            postfix_expr.push(operand_str);

            // Pop stack until an operator of equal precedence is found, append the popped operators to the postfix string
            // This is done to give priority for multiplication and division
	    if stack.len() == 0 {
	        stack.push(c);
		continue;
	    }
	    else {
		let mut s_top = stack.pop().unwrap();
		if s_top=='*' || s_top=='/' {
		    postfix_expr.push(s_top.to_string());
		}
		else {
		    stack.push(s_top);
		}
		while stack.len() > 0 {
		    s_top = stack.pop().unwrap();
		    if s_top=='*' || s_top=='/' {
			postfix_expr.push(s_top.to_string());
		    }
		    else {
			stack.push(s_top);
		        break;
		    }
		}
		stack.push(c);
	    }
	}
    }
    let mut operand_str = "".to_string();
    while !operand_stack.is_empty() {
        operand_str.push(operand_stack.pop_front().unwrap())
    }
    postfix_expr.push(operand_str);

    //Append all the remaining operators to the postfix string
    while stack.len() > 0 {
	postfix_expr.push((stack.pop().unwrap().to_string()));
    }
    //return postfix_expr.iter().cloned().collect::<String>();
    println!("{:?}", postfix_expr);
    return postfix_expr;
}

fn postfix_parse(expr: Vec<String>) -> i32{
    //Maintain a stack of terminals
    //let mut stack: Vec<i32> = Vec::new();
    let mut expr_stack = List::Nil;
    //expr_stack = push(expr_stack, 3);
    //let (d, expr_stack_tmp) = pop(expr_stack);
    //expr_stack = Rc::try_unwrap(expr_stack_tmp).unwrap();
    //println!("{:?}", d.unwrap());
    for c in expr.iter() {
        //Push terminals onto stack after type casting them to i32
	if c.chars().nth(0).unwrap() >='0' && c.chars().nth(0).unwrap() <='9' {
	        expr_stack = push(expr_stack, i32::from_str(c).unwrap());
	//      push_char_onto_stack(expr_stack,8);  
        }
	else if c=="+" || c=="*" || c=="-" || c=="/" {

	let (d, stack) = pop(expr_stack);
	expr_stack = Rc::try_unwrap(stack).unwrap();
	let e1 = d.unwrap();

	let (d, stack) = pop(expr_stack);
	expr_stack = Rc::try_unwrap(stack).unwrap();
	let e2 = d.unwrap();

	//println!("{:?}",e1);
	//}
	    if c=="+" {
 	        expr_stack = push(expr_stack, e1 + e2);
            }
	    else if c=="*" {
	        expr_stack = push(expr_stack, e1 * e2);
	    }
	    else if c=="/" {
	        expr_stack = push(expr_stack, e2/e1);
	    }
            else if c=="-" {
	        expr_stack = push(expr_stack, e2 - e1);
	    }
        }
    } 

    if length(&expr_stack) > 1 {
	println!("Check expression");
	return 1;
    }
    else {
	let (d, stack) = pop(expr_stack);
	return d.unwrap();
   }
}

fn main() {
    println!("Enter expression in infix format");
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok().expect("Failed to parse input");
    let postfix_expr = infix_to_postfix(input);
    println!("{}", postfix_parse(postfix_expr));
}
