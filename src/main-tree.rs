// extern crate rand;
use rand::Rng;
use std::char;
use std::cmp;

// In this program, all subtyping is done via SUM TYPES (enums, enums in structs, ...).
// It's a pretty good solution formally, but the Rust syntax for enums and structs can get kind of verbose.
// The other (possibly more common) solution would be inheritance, but Rust doesn't implement it.
// Traits work in a very similar way to inheritance, but they lack some features, especially the ability 
// to have fields in traits. Technically you can work around this with get and set functions in the parent class,
// but those would need to have be implemented identically a bunch of times in each derived class. 



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
    // Times, 
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
const SPLITTERS: [Token; 6] = [
    Token::Hit,
    
    Token::Addition, 
    Token::Subtraction,
    
    Token::Division,
    Token::Multiplication,
    // Token::Times,

    // ...
    Token::DiceRoll,
];



// fn fill(slot: &mut Box<NodeSlot>) {
//     match &mut slot.node {
//         NodeType::Binary(a) => {

//             fill(&mut a.leftleaf);
//             fill(&mut a.rightleaf);

//             let left = &mut a.leftleaf;
//             let right = &mut a.rightleaf;

//             let leftwidth = left.pos.width.unwrap();
//             let rightwidth = left.pos.width.unwrap();
//             slot.pos.width = Some(leftwidth + rightwidth + 1 );
//             let wstar = cmp::min(leftwidth, rightwidth);
//             let astar;
//             astar = (wstar + 1)/2;
            

//             left.pos.x = Some(0 - astar);

//             right.pos.x = Some(astar);
//             left.pos.y = Some(0 - astar);
//             right.pos.y = Some(0 - astar);



//             let leftres = left.noderesult.unwrap(); // unwrap checks can be done here
//             let rightres = right.noderesult.unwrap();
            
//             match a.binode {
//                 BinaryNodeType::Addition(_) => {
//                     let crit = std_crit_combine(leftres.crit, rightres.crit);
//                     let res = leftres.value + rightres.value;
//                     slot.noderesult = Some(DiceResult{ value: res, crit: crit});
//                 }

//                 BinaryNodeType::Subtraction(_) => {
//                     let crit = std_crit_combine(leftres.crit, rightres.crit);
//                     let res = leftres.value - rightres.value;
//                     slot.noderesult = Some(DiceResult{ value: res, crit: crit});
//                 }

//                 BinaryNodeType::Multiplication(_) => {
//                     let crit = std_crit_combine(leftres.crit, rightres.crit);
//                     let res = leftres.value * rightres.value;
//                     slot.noderesult = Some(DiceResult{ value: res, crit: crit});
//                 }

//                 BinaryNodeType::Division(_) => {
//                     let crit = std_crit_combine(leftres.crit, rightres.crit);
//                     let res = leftres.value / rightres.value;
//                     slot.noderesult = Some(DiceResult{ value: res, crit: crit});
//                 }

//                 BinaryNodeType::DiceRoll(_) => {
//                     let mut n = 0;
//                     let imax = leftres.value;
//                     for _i in 0..imax {
//                         n += rand::thread_rng().gen_range(1, rightres.value + 1);
//                     }
//                     let crit;
//                     if rightres.value == 20 && n == 20 {
//                         crit = Crit::NatMax;
//                     } else if n == 1 {
//                         crit = Crit::NatMin;
//                     } else {
//                         crit = Crit::Normal;
//                     }
                    
//                     slot.noderesult = Some(DiceResult{ value: n, crit: crit })
//                 }
                
//                 BinaryNodeType::Hit(_) => {
//                     let n;
//                     if leftres.crit == Crit::NatMax || rightres.crit == Crit::NatMax {
//                         n = 1;
//                     }else if leftres.value >= rightres.value {
//                         n = 1;
//                     } else {
//                         n = 0;
//                     }

//                     slot.noderesult = Some(DiceResult{value: n, crit: Crit::Normal});
//                 }

//                 // BinaryNodeType::Times(_) => {
//                 //     // This implementation will look like cheating, and it won't mesh that good with the 
//                 //     // whole "filling" metaphor.
//                 //     // this is because Times is not really a binary operator.
//                 //     // At best, it's a short-hand for a n-ary operator with n equal branches.
//                 //     // it fills its right branch many times, but only the last one will remain in the noderesult.

//                 //     let mut res = rightres.value;
//                 //     for _i in 0..(leftres.value - 1) {
//                 //         fill(&mut a.rightleaf);            
//                 //         let rightres = right.noderesult.unwrap();
//                 //         res += rightres.value;
//                 //     }
//                 //     // Times forgets about crit status of singular rolls. 
//                 //     // You should probably have used hit/pass before Times 
//                 //     slot.noderesult = Some(DiceResult{ value: res, crit: Crit::Normal}) ;
        
//                 // }

//             }

//         }

//         NodeType::Zero(a) => {
//             match &a.zeronode {
//                 ZeroNodeType::Number(_) => {
//                     // for Numbers, the noderesult and the width is hard-coded at build time.
//                 }
//             }
//         }
//     }



// }

// fn draw_2D_vec(startslot: &mut Box<NodeSlot>) -> Vec<Vec<char>> {
//     let mut v = vec![vec!['.'; 30]; 20];
//     let startx = 15;
//     let starty = 19;

//     fn dive(slot: &Box<NodeSlot>, v: &mut Vec<Vec<char>>, lastx: i32, lasty: i32) {
//         let this_rel_x = slot.pos.x.unwrap()*2;
//         let this_rel_y = slot.pos.y.unwrap()*2;

//         let this_w = slot.pos.width.unwrap();
    
//         let newx = lastx + this_rel_x;
//         let newy = lasty + this_rel_y;
//         let number = slot.noderesult.unwrap().value as u32;
//         let nchar = char::from_digit(number, 10u32).unwrap();
//         v[(newy) as usize][(newx) as usize] =  nchar;

//         match &slot.node {
//             NodeType::Binary(a) => {
//                 for l in 1..(this_w) {
//                     v[(newy - l) as usize][(newx + l) as usize] = '/';
//                     v[(newy - l) as usize][(newx - l) as usize] = '\\';
//                 }
//                 let opchar = match a.binode {
//                     BinaryNodeType::Addition(_) => '+',
//                     BinaryNodeType::Subtraction(_) => '-',
//                     BinaryNodeType::Multiplication(_) => '*',
//                     BinaryNodeType::Division(_) => '/',
//                     BinaryNodeType::DiceRoll(_) => 'd',
//                     BinaryNodeType::Hit(_) => 'h',
                    
//                 };
//                 v[(newy - this_w +1 ) as usize][(newx) as usize] =  opchar;
//             },
//             NodeType::Zero(_) => { },
//         }




//         //recurs dive
//         match &slot.node {
//             NodeType::Binary(a) => {
//                 dive(&a.leftleaf, v, newx, newy);
//                 dive(&a.rightleaf, v, newx, newy);
//             },
//             NodeType::Zero(_) => {},
//         }
//     }

//     startslot.pos.x = Some(0);
//     startslot.pos.y = Some(0);
//     dive(&startslot, &mut v, startx, starty);

//     return v;
// }

fn print_2D_vec(v: Vec<Vec<char>>) {
    for i in v {
        for j in i {
            print!("{}",j);
        }
        print!("\n");
    }
}

struct Tree {
    heap: Vec<NodeSlot>,
    root: NodeSlot,
}
impl Tree {
    fn build(tokens: &Vec<Token>) -> Self {
        let t_heap = Vec::<NodeSlot>::with_capacity(20);
        let dummy_root = build_number(99);
        let tree = Tree{ heap: t_heap, root: dummy_root };

        return tree;

        
        // let mut tree = Tree{ heap: Vec::with_capacity(20) };
        // let dummy_number = build_number(10);
        // let dummy_number2 = build_number(10);
        // let dummy_number3 = build_number(10);
    
        // let leftside = Box::new(dummy_number);
        // let rightside = Box::new(dummy_number2);
        // let innerinner = BinaryNodeType::Addition(AdditionNode{});
        // let inner = BinaryNodeSlot{ leftleaf: leftside, rightleaf: rightside, binode: innerinner, };
        // let dummy_add = ( NodeSlot{ pos: EMPTYPOS, noderesult: None, node: NodeType::Binary(inner), } );
    
    
        // tree.heap.push(dummy_number3);
        // tree.heap.push(dummy_add);
        
        // return tree;

    }


    fn add_node_from_expr(&mut self, vec: &[Token]) -> usize {
        // figure out if the expression is of the form (...)+(...),
        // where (...) can also have no parentheses but a lower priority op, like in 4*5+4*5
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
                self.heap.push(  build_number(temp)    ); //might need to BOX after?
                return self.heap.len() -1; // returns the index to the last added element
    
            }else{
                // eprintln!("no splitter found, got to clear parentheses maybe"); //PARSING DEBUG
    
                if let Token::LeftPar = vec[0] { 
                    return Tree::add_node_from_expr(self, &vec[1..vec.len()-1]);
                } else {
                    return Tree::add_node_from_expr(self, &vec); //TODO clear this
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
    
            let currlen = self.heap.len();
            self.heap.push(Tree::build_node_from_splitter(self, vec, splitter, poz));
            return currlen; // returns the index to the added element
    
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

    // from a string of tokens in which "splitter" is the one with top priority,
    // a node is built for the splitter token. its leaves will be built afterwards, by recursively parsing the 
    // remaining parts of the string, left and right of the splitter.
    fn build_node_from_splitter(&mut self, vec: &[Token], splitter: &Token, poz: usize) -> NodeSlot {
        
        
        
        match splitter {
            Token::DiceRoll => {
                // d6 is short for 1d6 
                let innerinner = BinaryNodeType::DiceRoll(DiceRollNode{});
                let rightside = Tree::add_node_from_expr(self, &vec[poz+1..]);
                let leftside;
                if poz == 0 {
                    self.heap.push(build_number(1));
                    leftside = self.heap.len() - 1;

                } else {
                    leftside = Tree::add_node_from_expr(self, &vec[0..poz]);
                }

                let inner = BinaryNodeSlot{ leftleaf: leftside, rightleaf: rightside, binode: innerinner, };
                return  NodeSlot{ pos: EMPTYPOS, noderesult: None, node: NodeType::Binary(inner), } ;
            },

            // Token::Times => {
            //     let leftside = Tree::add_node_from_expr(self, &vec[0..poz]);
            //     let rightside = Tree::add_node_from_expr(self, &vec[poz+1..]);
            //     // as default, (A) x (B) means "roll A copies of B", 
            //     // except if A is a long expression and B is a single number, like in (4d6 + 5)x3: 
            //     // in this care it's B copies of A (3 copies of the parenthesis).
            //     let innerinner = BinaryNodeType::DiceRoll(DiceRollNode{});
            //     let inner;
            //     if vec[poz+1..].len() == 1 && vec[0..poz].len() != 1 {
            //         if let Token::Number(num) = vec[poz+1] {
            //             println!("special case with number {} on the right", num);
            //             inner = BinaryNodeSlot{ leftleaf: leftside, rightleaf: rightside, binode: innerinner, };
            //         } else {
            //             panic!("bad error? sequence long 1 but is not a number?");
            //         }
            //     }else{
                    
            //         inner = BinaryNodeSlot{ leftleaf: rightside, rightleaf: leftside, binode: innerinner, };
            //     }

            //     return Box::new( NodeSlot{ pos: None, noderesult: None, node: NodeType::Binary(inner), } );
            
            // },

            _ => { },
        }
        

        // simple cases, with the recursive parsing grouped outside of the match
        
        let leftside = Tree::add_node_from_expr(self, &vec[0..poz]);
        let rightside = Tree::add_node_from_expr(self, &vec[poz+1..]);
        
        let innerinner;
        
        match splitter {
            Token::Addition => {
                innerinner = BinaryNodeType::Addition(AdditionNode{});
            },
            Token::Multiplication => {
                innerinner = BinaryNodeType::Multiplication(MultiplicationNode{});
            },
            
            Token::Subtraction => {
                innerinner = BinaryNodeType::Subtraction(SubtractionNode{});
            },
            Token::Division => {
                innerinner = BinaryNodeType::Division(DivisionNode{});
            },
            Token::Hit => {
                innerinner = BinaryNodeType::Hit(HitNode{});
            },
            _ => { panic!("error: trying to split with a token that shouldn't split")}
        }

        let inner = BinaryNodeSlot{ leftleaf: leftside, rightleaf: rightside, binode: innerinner, };
        return NodeSlot{ pos: EMPTYPOS, noderesult: None, node: NodeType::Binary(inner), } ;



    }

    
}

// LITERAL TYPE THEORY

struct NodeSlot {
    pos: NodePositionInfo,
    noderesult: Option<DiceResult>,
    node: NodeType,
}

// divided by number of leaves
enum NodeType { 
    Zero(ZeroNodeSlot),
    // Unary(UnaryNode),
    Binary(BinaryNodeSlot),
    // ...
}

struct BinaryNodeSlot {
    leftleaf: usize,
    rightleaf: usize,
    binode: BinaryNodeType, 
}
struct ZeroNodeSlot {
    zeronode: ZeroNodeType,
}

enum BinaryNodeType {
    Addition(AdditionNode),
    Subtraction(SubtractionNode),
    Multiplication(MultiplicationNode),
    // Times(TimesNode),
    Division(DivisionNode),
    DiceRoll(DiceRollNode),
    Hit(HitNode),
}
enum ZeroNodeType {
    Number(NumberNode),
}

struct AdditionNode { }
struct SubtractionNode { }
struct MultiplicationNode { }
// struct TimesNode { }
struct DivisionNode { }
struct DiceRollNode { }
struct HitNode { }
struct NumberNode { value: i32, }

fn build_number(val: i32) -> NodeSlot {
    let innerinner = ZeroNodeType::Number( NumberNode{value: val} );
    let inner = ZeroNodeSlot{ zeronode: innerinner };
    let res = DiceResult{ value: val, crit: Crit::Normal };
    let mut tmppos = EMPTYPOS; tmppos.width = Some(1);
    return NodeSlot{ pos: tmppos, noderesult: Some(res), node: NodeType::Zero(inner) }
}





#[derive(PartialEq)]
#[derive(Debug, Copy, Clone)]
enum Crit {
    Normal,
    NatMax,
    NatMin,
}

fn std_crit_combine(crit1: Crit, crit2: Crit) -> Crit {
    if crit1 == Crit::NatMax || crit2 == Crit::NatMax {
        return Crit::NatMax;            
    }
    else if crit1 == Crit::NatMin || crit2 == Crit::NatMin {
        return Crit::NatMin;
    }  else {
        return Crit::Normal;
    }
    // note that the case NatMin + NatMax hits the first condition first, 
    // so it gives NatMax. Usually you don't sum together d20's at all, so it shouldn't matter.

}


#[derive(Debug, Copy, Clone)]
struct DiceResult {
    value: i32,
    crit: Crit,
}

struct NodePositionInfo { 
    x: Option<i32>, // relative to parent!
    y: Option<i32>,
    width: Option<i32>, // width of downwards triangle that has this node as the bottom tip
}
const EMPTYPOS: NodePositionInfo = NodePositionInfo{ x: None, y: None, width: None };


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
            // 'x' => { res.push( Token::Times ); },
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




use std::io::{self};



fn main() -> io::Result<()> {

    // loop {    let mut expression = String::new();
    //     match io::stdin().read_line(&mut expression) {
    //         Ok(_n) => {

    //         }
    //         Err(error) => println!("error: {}", error),
    //     }
    //     let expression = expression.trim();
    //     let mut tokens = to_tokens(&expression);
    //     let tokens = clear_spaces(&mut tokens);
    //     println!("{:?}", tokens);
    //     let mut mola = parse_expression(&tokens);
    //     fill(&mut mola);
    //     println!("result: {}", mola.noderesult.unwrap().value);
    //     let arr = draw_2D_vec(&mut mola);
    //     print_2D_vec(arr);
        
    // }

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