use std::io;
mod funlist;
use funlist::*;

fn infix_to_postfix(input: List<char>) -> List<List> {
    let mut postfix = List::Nil;
    let mut operators = List::Nil;
    let mut temp_operand = List::Nil;

    ///'''
    /// Match all chars in 'input'
    ///     If c is a digit, then push onto a Digit list temporarily until all digits in an operand are read (after which add the operand to the Postfix list)
    ///     If c is an operator, pop everything of higher predence and add it to postfix list, then add c to list of Operators
    ///'''
    postfix
}

fn evaluate_postfix(input: List<List>) -> i32 {

    ///'''
    /// Convert list to number and apply operation on two at a time
    ///'''

}

fn main() {
    println!("Enter expression in infix format");
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok().expect("Failed to parse input");
  
    /// '''
    /// Convert string to List<char> (reverse order)
    /// '''

    let postfix_expr = infix_to_postfix(input_list);
    let result = evaluate_postfix(postfix_expr);
}

