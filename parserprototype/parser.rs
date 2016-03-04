use std::io;

fn infix_to_postfix(expr: String) -> String {

    // Maintain a stack of operators and postfix string of operands
    let mut stack: Vec<char> = Vec::new();
    let mut postfix_expr: Vec<char> = Vec::new();

    for c in expr.chars() {

	if c>'0' && c<'9' {
	    // Append terminal to postfix string
	    postfix_expr.push(c);
        }

	else if c=='*' {
	    // Pop stack until an operator of lower precedence is found, append the popped operators to the postfix string
	    // This is done to give priority for division over multiplication
            if stack.len() == 0 {
                stack.push(c);
                continue;
            }
            else {
                let mut s_top = stack.pop().unwrap();
                if s_top=='*' || s_top=='/' {
                    postfix_expr.push(s_top);
                }
                else {
                    stack.push(s_top);
                }
                while stack.len() > 0 {
                    s_top = stack.pop().unwrap();
                    if s_top=='*' || s_top=='/' {
                        postfix_expr.push(s_top);
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
	   stack.push(c);
        }

	else if c=='+' || c=='-' {
            // Pop stack until an operator of equal precedence is found, append the popped operators to the postfix string
            // This is done to give priority for multiplication and division
	    if stack.len() == 0 {
	        stack.push(c);
		continue;
	    }
	    else {
		let mut s_top = stack.pop().unwrap();
		if s_top=='*' || s_top=='/' {
		    postfix_expr.push(s_top);
		}
		else {
		    stack.push(s_top);
		}
		while stack.len() > 0 {
		    s_top = stack.pop().unwrap();
		    if s_top=='*' || s_top=='/' {
			postfix_expr.push(s_top);
		    }
		    else {
			stack.push(s_top);
		        break;
		    }
		}
		stack.push(c);
	    }
	}
	else if c=='(' {
	    stack.push(c);
	}
	else if c==')' {
	    let mut s_top = stack.pop().unwrap();
	    while s_top != '(' {
	        postfix_expr.push(s_top);
		s_top = stack.pop().unwrap();
	    }
	}
    }
    //Append all the remaining operators to the postfix string
    while stack.len() > 0 {
	postfix_expr.push(stack.pop().unwrap());
    }
    return postfix_expr.iter().cloned().collect::<String>();;
}

fn postfix_parse(expr: String) -> i32{
    //Maintain a stack of terminals
    let mut stack: Vec<i32> = Vec::new();    

    for c in expr.chars(){
        //Push terminals onto stack after type casting them to i32
	if c>='0' && c<='9' {
	    stack.push(c as i32 - '0' as i32);
        } 

	else if c=='+' || c=='*' || c=='-' || c=='/' {
	    let e1 = stack.pop().unwrap();
	    let e2 = stack.pop().unwrap();
	    if c=='+' {
 	        stack.push(e1 + e2);
            }
	    else if c=='*' {
	        stack.push(e1 * e2);
	    }
	    else if c=='/' {
	        stack.push(e2/e1);
	    }
	    else if c=='-' {
	        stack.push(e2 - e1);
	    }
        }
    }
    if stack.len() > 1 {
	println!("Check expression");
	return 1
    }
    return stack.pop().unwrap();    
}

fn main() {
    println!("Enter expression in infix format");
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok().expect("Failed to parse input");
    let postfix_expr = infix_to_postfix(input);
    println!("{}", postfix_parse(postfix_expr));

}

