import sys

class Parser():
	def infix_to_postfix(self, expr):
	    stack = list()
	    postfix_expr = list()
	    for c in expr:
	 	if c > '0' and c < '9':
			#Add terminal to the postfix expression
			postfix_expr.append(c)
		if c=='*' or c=='/':
			#Higher precedence operators are pushed onto the stack
			stack.append(c)
		if c=='+' or c=='-':
			#Pop out operators with higher precedence from the stack and append it to the postfix expression, then push lower order operators on the stack
			if len(stack) == 0:
				stack.append(c)
				continue
			else:
			        s_top=stack.pop()
				if(s_top=='*' or s_top=='/'):
					postfix_expr.append(s_top)
				else:
					stack.append(s_top)
				while( len(stack) >= 0 ):
					s_top=stack.pop()
					if(s_top=='*' or s_top=='/'):
						postfix_expr.append(s_top)
					else:
						stack.append(s_top)
						break
				stack.append(c)
	    while(len(stack) is not 0):
		postfix_expr.append(stack.pop())
	    return postfix_expr
	

	def postfix_parse(self, expr):
		stack = list()
		for c in expr:
	            if c >'0' and c <'9':
                        #Add terminal to stack
		        stack.append(int(c))
		    elif c=='+':
	                #Pop top two values on the stack, evaluate the subexpression and push the result back onto the stack
		  	stack.append(stack.pop()+stack.pop())
		    elif c=='*':
			stack.append(stack.pop()*stack.pop())
		    elif c=="/":
			den = stack.pop()
			stack.append(stack.pop()/den)
		    elif c=='-':
			stack.append(stack.pop() - stack.pop())

		#If the number of terminals exceed the operators, throw an error
		if len(stack) != 1:
			print 'Check expression'
			return 1
		#Print the evaluated expression
		return stack[0]

print 'Enter expression in infix format'
inp_expr = sys.stdin.readline()
parser = Parser()
postfix_expr = parser.infix_to_postfix(inp_expr)
print parser.postfix_parse(postfix_expr)
