Common:
* refactor lexer: conditions on iterator's end 
* tests 
* make parser and lexer consume or borrow String|Token, not .clone()
* refactor way the parser panics  
--- 
* make code_generator generate correct programs 

Parser:
* remove using self.current outside utils funcs
* think about self.current : should cycle starts or ends with self.advance()