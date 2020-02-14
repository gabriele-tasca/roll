// extern crate rand;
use rand::Rng;
use std::rc::Rc;
use std::cell::Cell;

// To implement a new function: 
// 1) Add a new value in the Token enum
// 2) Add the new Token in the priority array, and adjust the length in the type signature???
// 3) Add the parsing rule in the CHAR -> TOKEN BLOCK
// 4) Add the new Node struct with the Node method and the constructor
// 5) Write the linking between Token and Node in the TOKEN -> NODE BLOCK

// ... a little long maybe???


// Token list 
#[derive(Debug)]
#[derive(PartialEq)]
enum Token {
    LeftPar,
    RightPar,
    DiceRoll,
    Times, 
    Division,
    Multiplication,
    Addition,
    Subtraction,
    Hit, 
    Spaces(i32),
    Number(i32),
}

// enum OpType {
//     Both,
//     Left,
//     Right,
//     Zero,
// }

// struct LeftPar {
//     const 
// }


// priority array:
// the order here defines the execution priority 
const SPLITTERS: [Token; 7] = [
    Token::Hit,
    
    Token::Addition, 
    Token::Subtraction,
    
    Token::Division,
    Token::Multiplication,
    Token::Times,

    // ...
    Token::DiceRoll,
];


//LITERAL LINKING BLOCK
// each token is linked to a build function that builds the correct node.
// should this not work anymore, the old TOKEN -> NODE BLOCK is still there somewhere.
fn match_node_to_token(vec: &[Token], splitter: &Token, poz: usize) -> Rc<dyn Node> {

    return match splitter {
        // TOKEN -> NODE BLOCK

        Token::Addition => { AdditionNode::build(vec, poz) },
        Token::Multiplication => { MultiplicationNode::build(vec, poz) },
        Token::DiceRoll => { DiceRollNode::build(vec, poz) },            
        Token::Subtraction => { SubtractionNode::build(vec, poz) },
        Token::Division => { DivisionNode::build(vec, poz) },
        Token::Times => { TimesNode::build(vec, poz) },
        Token::Hit => { HitNode::build(vec, poz) },
        _ => { panic!("this splitter is not in the SPLITTER constant array!")}
    }

}


trait Node {
    fn eval(&self) -> DiceResult;
    fn get_pos_info(&self) -> Option<NodePositionInfo>;
    // eval returns the result but also annotates the partial results and position info in each node's fields.
}



struct AdditionNode {
    // root: Weak<dyn Node>,
    leftleaf: Rc<dyn Node>,
    rightleaf: Rc<dyn Node>,
    noderesult: Cell<Option<DiceResult>>,
    pos_info: Option<NodePositionInfo>,
}
impl AdditionNode {
    fn build(vec: &[Token], poz: usize) -> Rc<Self> {
        let leftside = parse_expression(&vec[0..poz]);
        let rightside = parse_expression(&vec[poz+1..]);
        return Rc::new( AdditionNode{ leftleaf: leftside, rightleaf: rightside, noderesult: Cell::new(None), pos_info: None});
    }
}


impl Node for AdditionNode {
    fn eval(&self) -> DiceResult {
        let lefteval = self.leftleaf.eval();
        let righteval = self.rightleaf.eval();
        // if one of the numbers comes from a critical dice roll, the result will be marked as critical
        // the case critical success + critical failure is arbitrarily set to result in critical success
        // usually this won't matter.  
        let crit;
        if lefteval.crit == Crit::NatMin || righteval.crit == Crit::NatMin {
            crit = Crit::NatMin;
        } else if lefteval.crit == Crit::NatMax || righteval.crit == Crit::NatMax {
            crit = Crit::NatMax;            
        } else {
            crit = Crit::Normal;
        }
        let sum = lefteval.value + righteval.value;
        
        let rez = DiceResult{ value: sum, crit: crit};
        self.noderesult.set(Some(rez));
        return rez;
    }
    fn get_pos_info(&self) -> Option<NodePositionInfo> {
        return self.pos_info;
    }
    
}

/////////////
struct SubtractionNode {
    // root: Weak<dyn Node>,
    leftleaf: Rc<dyn Node>,
    rightleaf: Rc<dyn Node>,
    noderesult: Cell<Option<DiceResult>>,
}
impl SubtractionNode {
    fn build(vec: &[Token], poz: usize) -> Rc<Self> {

        let leftside = parse_expression(&vec[0..poz]);
        let rightside = parse_expression(&vec[poz+1..]);
        return Rc::new( SubtractionNode{ leftleaf: leftside, rightleaf: rightside, noderesult: Cell::new(None), });
    }
}
impl Node for SubtractionNode {
    fn eval(&self) -> DiceResult {
        let lefteval = self.leftleaf.eval();
        let righteval = self.rightleaf.eval();
        // if one of the numbers comes from a critical dice roll, the result will be marked as critical
        // the case crit success - crit failure is arbitrarily set to result in crit success
        // usually this won't matter.  
        let crit;
        if lefteval.crit == Crit::NatMin || righteval.crit == Crit::NatMin {
            crit = Crit::NatMin;
        } else if lefteval.crit == Crit::NatMax || righteval.crit == Crit::NatMax {
            crit = Crit::NatMax;            
        } else {
            crit = Crit::Normal;
        }
        let sum = lefteval.value - righteval.value;
        
        let rez = DiceResult{ value: sum, crit: crit};
        self.noderesult.set(Some(rez));
        return rez;
    }
}

/////////////
struct MultiplicationNode {
    // root: Weak<dyn Node>,
    leftleaf: Rc<dyn Node>,
    rightleaf: Rc<dyn Node>,
        noderesult: Cell<Option<DiceResult>>,

}
impl MultiplicationNode {
    fn build(vec: &[Token], poz: usize) -> Rc<Self> {
        let leftside = parse_expression(&vec[0..poz]);
        let rightside = parse_expression(&vec[poz+1..]);
        return Rc::new( MultiplicationNode{ leftleaf: leftside, rightleaf: rightside, noderesult: Cell::new(None), });
    }
}
impl Node for MultiplicationNode {
    fn eval(&self) -> DiceResult {
        let lefteval = self.leftleaf.eval();
        let righteval = self.rightleaf.eval();
        // if one of the numbers comes from a critical dice roll, the result will be marked as critical
        // the case critical success * critical failure is arbitrarily set to result in critical success
        // usually this won't matter.  
        let crit;
        if lefteval.crit == Crit::NatMin || righteval.crit == Crit::NatMin {
            crit = Crit::NatMin;
        } else if lefteval.crit == Crit::NatMax || righteval.crit == Crit::NatMax {
            crit = Crit::NatMax;            
        } else {
            crit = Crit::Normal;
        }
        let sum = lefteval.value * righteval.value;
        
        let rez = DiceResult{ value: sum, crit: crit};
        self.noderesult.set(Some(rez));
        return rez;
    }
}

/////////////
struct TimesNode {
    // root: Weak<dyn Node>,
    number: Rc<dyn Node>,
    expr: Rc<dyn Node>,
        noderesult: Cell<Option<DiceResult>>,

}
impl TimesNode {
    fn build(vec: &[Token], poz: usize) -> Rc<Self> {

        let leftside = parse_expression(&vec[0..poz]);
        let rightside = parse_expression(&vec[poz+1..]);
        // as default, (A) x (B) means "roll A copies of B", 
        // except if A is a long expression and B is a single number, like in (4d6 + 5)x3: 
        // in this care it's B copies of A (3 copies of the parenthesis).
        if vec[poz+1..].len() == 1 && vec[0..poz].len() != 1 {
            if let Token::Number(num) = vec[poz+1] {
                println!("special case with number {} on the right", num);
                return Rc::new( TimesNode{ expr: leftside, number: rightside, noderesult: Cell::new(None), });
            } else {
                panic!("bad error");
            }
        }else{

            return Rc::new( TimesNode{ number: leftside, expr: rightside, noderesult: Cell::new(None), });
        }

    }
}
impl Node for TimesNode {
    // Times forgets about crit status of singular rolls. You should probably have used hit/pass before Times 
    fn eval(&self) -> DiceResult {
        let lefteval = self.number.eval();
        let righteval = self.expr.eval();
        let mut res = 0;
        for _i in 0..lefteval.value {
            res += righteval.value;
        }

        
        let rez = DiceResult{ value: res, crit: Crit::Normal};
        self.noderesult.set(Some(rez));
        return rez;
    }
}

/////////////
struct DivisionNode {
    // root: Weak<dyn Node>,
    leftleaf: Rc<dyn Node>,
    rightleaf: Rc<dyn Node>,
        noderesult: Cell<Option<DiceResult>>,

}
impl DivisionNode {
    fn build(vec: &[Token], poz: usize) -> Rc<Self> {
        let leftside = parse_expression(&vec[0..poz]);
        let rightside = parse_expression(&vec[poz+1..]);
        return Rc::new( DivisionNode{ leftleaf: leftside, rightleaf: rightside, noderesult: Cell::new(None), });
    }
}
impl Node for DivisionNode {
    fn eval(&self) -> DiceResult {
        let lefteval = self.leftleaf.eval();
        let righteval = self.rightleaf.eval();
        // if one of the numbers comes from a critical dice roll, the result will be marked as critical
        // the case critical success / critical failure is arbitrarily set to result in critical success
        // usually this won't matter.  
        let crit;
        if lefteval.crit == Crit::NatMin || righteval.crit == Crit::NatMin {
            crit = Crit::NatMin;
        } else if lefteval.crit == Crit::NatMax || righteval.crit == Crit::NatMax {
            crit = Crit::NatMax;            
        } else {
            crit = Crit::Normal;
        }

        // in rust rounded-down integer division is auto when using i32's
        let sum = lefteval.value / righteval.value;
        
        let rez = DiceResult{ value: sum, crit: crit};
        self.noderesult.set(Some(rez));
        return rez;
    }
}

/////////////
struct DiceRollNode {
    // root: Weak<dyn Node>,
    dicenumber: Rc<dyn Node>,
    dicesize: Rc<dyn Node>,
        noderesult: Cell<Option<DiceResult>>,

}
impl DiceRollNode {
    fn build(vec: &[Token], poz: usize) -> Rc<Self> {
        if poz == 0 { // d6 is short for 1d6
            let leftside = Rc::new(NumberNode::build(1));
            let rightside = parse_expression(&vec[poz+1..]);
            return Rc::new( DiceRollNode{ dicenumber: leftside, dicesize: rightside, noderesult: Cell::new(None), });
        } else {                    
            let leftside = parse_expression(&vec[0..poz]);
            let rightside = parse_expression(&vec[poz+1..]);
            return Rc::new( DiceRollNode{ dicenumber: leftside, dicesize: rightside, noderesult: Cell::new(None), });
        }
    }
}
impl Node for DiceRollNode {
    fn eval(&self) -> DiceResult {
        let numbereval = self.dicenumber.eval();
        let sizeeval = self.dicesize.eval();
        let mut n = 0;
        let imax = numbereval.value;
        for _i in 0..imax {
            n += rand::thread_rng().gen_range(1, sizeeval.value + 1);
        }
        let crit;
        if sizeeval.value == 20 && n == 20 {
            crit = Crit::NatMax;
        } else if n == 1 {
            crit = Crit::NatMin;
        } else {
            crit = Crit::Normal;
        }
        
        let rez = DiceResult{ value: n, crit: crit };
        self.noderesult.set(Some(rez));
        return rez;
    }
}

/////////
struct HitNode {
    // root: Weak<dyn Node>,
    to_hit: Rc<dyn Node>,
    armorclass: Rc<dyn Node>,
        noderesult: Cell<Option<DiceResult>>,

}
impl HitNode {
    fn build(vec: &[Token], poz: usize) -> Rc<Self> {
        if poz == 0 { // d6 is short for 1d6
            let leftside = Rc::new(NumberNode::build(1));
            let rightside = parse_expression(&vec[poz+1..]);
            return Rc::new( HitNode{ to_hit: leftside, armorclass: rightside, noderesult: Cell::new(None), });
        } else {                    
            let leftside = parse_expression(&vec[0..poz]);
            let rightside = parse_expression(&vec[poz+1..]);
            return Rc::new( HitNode{ to_hit: leftside, armorclass: rightside, noderesult: Cell::new(None), });
        }
    }
}
impl Node for HitNode {
    
    fn eval(&self) -> DiceResult {
        let hiteval = self.to_hit.eval();
        let armorclasseval = self.armorclass.eval();
        let n;
        if hiteval.crit == Crit::NatMax || armorclasseval.crit == Crit::NatMax {
            n = 1;
        }else if hiteval.value >= armorclasseval.value {
            n = 1;
        } else {
            n = 0;
        }

        
        let rez = DiceResult{value: n, crit: Crit::Normal};
        self.noderesult.set(Some(rez));
        return rez;
    }
}

/////////////
struct NumberNode {
    // root: Weak<dyn Node>,
    value: i32,
    noderesult: Cell<Option<DiceResult>>,

}
impl NumberNode {
    fn build(value: i32) -> Self {
        return NumberNode{value: value, noderesult: Cell::new(Some(DiceResult{value: value, crit: Crit::Normal}))  };
    }
}
impl Node for NumberNode {
    fn eval(&self) -> DiceResult {
        
        let rez = DiceResult{value: self.value, crit: Crit::Normal };
        // for numbers, the noderesult is hard-coded at build time.
        return rez;
    }
}


// Parsing functions ecc

#[derive(PartialEq)]
#[derive(Debug, Copy, Clone)]
enum Crit {
    Normal,
    NatMax,
    NatMin,
}


#[derive(Debug, Copy, Clone)]
struct DiceResult {
    value: i32,
    crit: Crit,
}

struct NodePositionInfo { 
    x: i32, // relative to parent!
    y: i32,
    width: i32, // width of downwards triangle that has this node as the bottom tip
}

struct NumberStack {
    number_stack: Vec<char>,
}
impl NumberStack {
    fn push(&mut self, x: char) {
        self.number_stack.push(x);
    }
    fn flush(&mut self) -> i32 {
        let mut y = 0;
        let ndigits = self.number_stack.len() as u32;
        const RADIX: u32 = 10;
        for (n, x) in self.number_stack.iter().enumerate(){
            
            y += (x.to_digit(RADIX).unwrap() as i32)  * 10_i32.pow(ndigits - (n as u32) -1);
        }
        self.number_stack.clear();
        return y;
    }
}

fn to_tokens(expr: &str) -> Vec<Token> { 
    
    let mut res: Vec<Token> = Vec::with_capacity(50);
    let mut number_stack = NumberStack { number_stack: Vec::new(), };

    let mut n = 0;
    let vecchar : Vec<_> = expr.chars().collect();

    // res.push(Token::Spaces(1)); // space padding, useful when converting spaces to parentheses?

    while n < vecchar.len() {  

        match vecchar[n] {
            '1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9'|'0' => {
                loop {
                    let x = vecchar.get(n);
                    match x {
                        Some('1')|Some('2')|Some('3')|Some('4')|Some('5')|Some('6')|Some('7')|Some('8')|Some('9')|Some('0') => { 
                            number_stack.push(*x.unwrap()); 
                        },
                        _ => { 
                            res.push(Token::Number(number_stack.flush()));
                            n -= 1;
                            break;
                        },
                    }

                    n += 1;
                }
            
            },

            ' ' => {
                let mut number_of_spaces = 0;
                loop {
                    let x = vecchar.get(n);
                    match x {
                        Some(' ') => { number_of_spaces += 1; },
                        _ => { 
                            res.push( Token::Spaces(number_of_spaces) );
                            n -= 1;
                            break;
                        },
                    }

                    n += 1;
                }
            
            },

            // matching the word "hit"
            // different command that start with h will need to be inside here...
            'h' => {
                if vecchar.get(n+1) == Some(&'i') && vecchar.get(n+2) == Some(&'t') {
                    res.push( Token::Hit );
                    n += 2;
                } 
            },

            // CHAR -> TOKEN BLOCK
            '(' => { res.push( Token::LeftPar ); },
            ')' => { res.push( Token::RightPar ); },
            'd' => { res.push( Token::DiceRoll ); },
            'x' => { res.push( Token::Times ); },
            '/' => { res.push( Token::Division ); },
            '*' => { res.push( Token::Multiplication ); },
            '+' => { res.push( Token::Addition ); },
            '-' => { res.push( Token::Subtraction ); },

            other => { println!("Malformed string: found {}", other) },
        }

        n += 1;
    }

    // res.push(Token::Spaces(1)); // space padding, useful when converting spaces to parentheses?

    return res;
}


fn clear_spaces(vec: &mut Vec<Token>) -> & Vec<Token> {
    // turn the select few into parentheses?
    // for (n, x) in vec.iter().enumerate() {        
        // match x {
        //     Spaces(_) => 
        // }
    // }

    // just kill them all lol
    vec.retain(|y| if let Token::Spaces(_) = y {false} else {true});
    return vec;
}


fn parse_expression(vec: &[Token]) -> Rc<dyn Node> {
    // figure out if the expression is of the form (...)+(...),
    // where (...) can also have no parentheses but a lower priority op, like 4*5
    // the case (...)+(...)+(...) works, but with two levels
    // eprintln!("parsing {:?}", vec); //PARSING DEBUG
    let mut par_balance: u8 = 0;
    let mut position_of_splitter: Option<usize> = None;
    'outer: for spl in &SPLITTERS{
        for (n, x) in vec.iter().enumerate() {        

            match x {
                Token::LeftPar => par_balance += 1,
                Token::RightPar => par_balance -= 1,
                other if other == spl => {

                    if par_balance == 0 {

                        position_of_splitter = Some(n);
                        break 'outer;
                    }
                }
                _ => {},              
            
            }
        }
    }
    // if no splitter is found, we have 3 cases: 
    // a number (base case)
    // (...), thus we parse recursively the ... without the parentheses
    // an expression without parentheses

    if let None = position_of_splitter {
        
        // single number case
        if vec.len() == 1 {
            // eprintln!("no splitter found, single number!"); //PARSING DEBUG
            let temp;
            if let Token::Number(i) = vec[0] {
                temp = i;
            }else{
                panic!("lol");
            }
            return Rc::new( NumberNode::build(temp)   );

        }else{
            // eprintln!("no splitter found, got to clear parentheses maybe"); //PARSING DEBUG

            if let Token::LeftPar = vec[0] { 
                return parse_expression(&vec[1..vec.len()-1]);
            } else {
                return parse_expression(&vec);
            }

        }

    } else {
        let poz = position_of_splitter.unwrap();
        let splitter = &vec[poz];

        // the match linking nodes to token is inside a function, so that implementing 
        // new operators requires mostly just additions to the headers and as 
        // little changes as possible to the body of the program. 
        // However, if any future operators will happen to have more complex building rules,
        // it might be hard to make them fit into this scheme. 
        // The commented code below is kept in case the scheme needs to be reversed. 

        return match_node_to_token(vec, splitter, poz);

        // match splitter {
        //     // TOKEN -> NODE BLOCK

        //     Token::Addition => {return AdditionNode::build(vec, poz); },
        //     Token::Multiplication => { return MultiplicationNode::build(vec, poz); },
        //     Token::DiceRoll => { return DiceRollNode::build(vec, poz); },            
        //     Token::Subtraction => { return SubtractionNode::build(vec, poz); },
        //     Token::Division => { return DivisionNode::build(vec, poz); },
        //     Token::Times => { return TimesNode::build(vec, poz); },
        //     _ => { panic!("this splitter is not in the SPLITTER constant array!")}
        // }

    }

}

use std::io::{self};

fn parse(expr: &str) -> Rc<dyn Node> {
    return parse_expression(clear_spaces(&mut to_tokens(expr)));
}


fn main() -> io::Result<()> {

    loop {    let mut expression = String::new();
        match io::stdin().read_line(&mut expression) {
            Ok(_n) => {

            }
            Err(error) => println!("error: {}", error),
        }
        let expression = expression.trim();
        let mut tokens = to_tokens(&expression);
        let tokens = clear_spaces(&mut tokens);
        // println!("{:?}", tokens);
        let mola = parse_expression(&tokens);
        println!("result: {}", mola.eval().value);
        
    }

    // FREQUENCY TEST
    // let big_number = 100000;
    // let expression = "1d20";  // MAKE THE DICE SIZE MATCH THE VEC! LEN
    // let mut counter = vec![0;20];
    // for _i in 0..big_number {
    //     let res = parse(expression).eval();
    //     counter[res as usize -1] += 1;
    // }

    // let expected = big_number/counter.len();
    // println!("expected: {}  times for each value", expected);
    // for i in 0..counter.len() {
    //     println!("value {} found  {} times, ", i+1, counter[i] )
    // }
    
    Ok(())
}