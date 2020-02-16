use rand::Rng;
use std::char;
use std::cmp;

// In this program, all subtyping is done via SUM TYPES (enums, enums in structs, ...).
// It's a pretty good solution formally, but the Rust syntax for enums and structs can get kind of verbose.
// The other (possibly more common) solution would be inheritance, but Rust doesn't implement it.
// Traits work in a very similar way to inheritance, but they lack some features, especially the ability 
// to have fields in traits. Technically you can work around this 
// with get and set functions for each field you'd want in the parent class,
// but those would need to have be implemented identically a bunch of times in each derived class. 



// To implement a new function: 
// 1) Add a new value in the Token enum
// 2) Add the new Token in the priority array, and adjust the length in the type signature (???)
// 3) Add the parsing rule in the CHAR -> TOKEN BLOCK
// 4) Add the new Node struct in the LITERAL TYPE THEORY block
// 5) Add a rule for building the node in the build_from_splitter function
// 6) Add a eval rule in the fill function
// 7) Update the draw function. Usually this means both set_pos and the ascii part




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


// from a string of tokens in which "splitter" is the one with top priority,
// a node is built for the splitter token. its leaves will be built afterwards, by recursively parsing the 
// remaining parts of the string, left and right of the splitter.
fn build_node_from_splitter(vec: &[Token], splitter: &Token, poz: usize) -> Box<NodeSlot> {
    
    
    // complex cases
    match splitter {

        Token::DiceRoll => {
            // d6 is short for 1d6 
            let innerinner = BinaryNodeType::DiceRoll(DiceRollNode{});
            let rightside = parse_expression(&vec[poz+1..]);
            let leftside;
            if poz == 0 {
                leftside = Box::new(  build_number(1)  );
            } else {
                leftside = parse_expression(&vec[0..poz]);
            }

            let inner = BinaryNodeSlot{ leftleaf: leftside, rightleaf: rightside, binode: innerinner, };
            return Box::new( NodeSlot{ pos: EMPTYPOS, noderesult: None, node: NodeType::Binary(inner), } );
        },

        Token::Times => {
            let n_times_string;
            let expr_string;
            // as default, (A) x (B) means "roll A copies of B".
            // except if A is a long expression and B is a single number, like in (4d6 + 5)x3: 
            // in this care it's B copies of A (3 copies of the parenthesis).
            if vec[poz+1..].len() == 1 && vec[0..poz].len() != 1 {
                if let Token::Number(num) = vec[poz+1] {
                    // n times is on the right
                    println!("special case with number {} on the right", num);
                    n_times_string = &vec[poz+1..];
                    expr_string = &vec[0..poz];
                } else {
                    panic!("bad error");
                }
            }else{
                // n times is on the left
                n_times_string = &vec[0..poz];
                expr_string = &vec[poz+1..];
            }

            let mut n_times_leaf = parse_expression(n_times_string);
            fill(&mut n_times_leaf);
            let n_times = n_times_leaf.noderesult.unwrap().value;

            let mut temp_v = Vec::with_capacity(n_times as usize + 1);
            temp_v.push(n_times_leaf); 

            for _j in 0..n_times {
                let jleaf = parse_expression(expr_string);
                temp_v.push(jleaf);
            }

            let innerinner = ManyNodeType::Times(TimesNode{});
            let inner = ManyNodeSlot{ leaves: temp_v, manynode: innerinner };
            return Box::new( NodeSlot{ pos: EMPTYPOS, noderesult: None, node: NodeType::Many(inner), } );


        }

        _ => { },
    }
    

    // simple cases, with the recursive parsing grouped outside of the match
    // it might a bad idea to put the parsing out of the match anyways,
    // because it still get called if the splitter is wrong!
    let leftside = parse_expression(&vec[0..poz]);
    let rightside = parse_expression(&vec[poz+1..]);
    
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
    return Box::new( NodeSlot{ pos: EMPTYPOS, noderesult: None, node: NodeType::Binary(inner), } );



}

fn fill(slot: &mut Box<NodeSlot>) {
    match &mut slot.node {
        NodeType::Binary(a) => {

            
            let left = &mut a.leftleaf;
            let right = &mut a.rightleaf;

            if left.noderesult == None {  fill(left); }
            if right.noderesult == None {  fill(right);  }

            let leftres = left.noderesult.unwrap(); // unwrap checks can be done here
            let rightres = right.noderesult.unwrap();
            
            match a.binode {
                BinaryNodeType::Addition(_) => {
                    let crit = std_crit_combine(leftres.crit, rightres.crit);
                    let res = leftres.value + rightres.value;
                    slot.noderesult = Some(DiceResult{ value: res, crit: crit});
                }

                BinaryNodeType::Subtraction(_) => {
                    let crit = std_crit_combine(leftres.crit, rightres.crit);
                    let res = leftres.value - rightres.value;
                    slot.noderesult = Some(DiceResult{ value: res, crit: crit});
                }

                BinaryNodeType::Multiplication(_) => {
                    let crit = std_crit_combine(leftres.crit, rightres.crit);
                    let res = leftres.value * rightres.value;
                    slot.noderesult = Some(DiceResult{ value: res, crit: crit});
                }

                BinaryNodeType::Division(_) => {
                    let crit = std_crit_combine(leftres.crit, rightres.crit);
                    let res = leftres.value / rightres.value;
                    slot.noderesult = Some(DiceResult{ value: res, crit: crit});
                }

                BinaryNodeType::DiceRoll(_) => {
                    let mut n = 0;
                    let imax = leftres.value;
                    for _i in 0..imax {
                        n += rand::thread_rng().gen_range(1, rightres.value + 1);
                    }
                    let crit;
                    if rightres.value == 20 && n == 20 {
                        crit = Crit::NatMax;
                    } else if n == 1 {
                        crit = Crit::NatMin;
                    } else {
                        crit = Crit::Normal;
                    }
                    
                    slot.noderesult = Some(DiceResult{ value: n, crit: crit })
                }
                
                BinaryNodeType::Hit(_) => {
                    let n;
                    if leftres.crit == Crit::NatMax || rightres.crit == Crit::NatMax {
                        n = 1;
                    }else if leftres.value >= rightres.value {
                        n = 1;
                    } else {
                        n = 0;
                    }

                    slot.noderesult = Some(DiceResult{value: n, crit: Crit::Normal});
                }

            }

        }

        NodeType::Zero(a) => {
            match &a.zeronode {
                ZeroNodeType::Number(_) => {
                    // for Numbers, the noderesult and the width is hard-coded at build time.
                }
            }
        }

        NodeType::Many(mslot) => {
            match &mslot.manynode {
                // ManyNodeType::ManySum(_) => {
                    
                //     let mut res = 0;
                //     for j in &mut mslot.leaves {
                //         if j.noderesult == None {  fill(j); }
                //         let jres = j.noderesult.unwrap();
                //         res += jres.value;
                //     }
                //     slot.noderesult = Some(DiceResult{ value: res, crit: Crit::Normal});
                //     // ^ change this maybe

                //     // none of the above has ever been tested
                    
                // }

                ManyNodeType::Times(_) => {
                    
                    let mut res = 0;
                    for j in &mut mslot.leaves[1..] {
                        if j.noderesult == None {  fill(j); }
                        let jres = j.noderesult.unwrap();
                        res += jres.value;
                    }
                    slot.noderesult = Some(DiceResult{ value: res, crit: Crit::Normal});

                }
            }
        }
        
    }



}


// from each root, set each leaf's position relative to the root, based on the leaves' width.
fn set_pos(slot: &mut Box<NodeSlot>) {
    const MIN_SPACING: i32 = 1;

    match &mut slot.node {
        NodeType::Binary(a) => {
            
            let left = &mut a.leftleaf;
            let right = &mut a.rightleaf;
            
            if left.pos.h == None {  set_pos(left); }
            if right.pos.h == None {  set_pos(right);  }
            
            // special cases: little cones

            // if both are cones:
            if left.pos.tipwidth_going_east.unwrap() == 0 && left.pos.tipwidth_going_west.unwrap() == 0 
                    && right.pos.tipwidth_going_east.unwrap() == 0   && right.pos.tipwidth_going_west.unwrap() == 0 {

                // if one is a single number:
                if left.pos.h.unwrap() == 1 || right.pos.h.unwrap() == 1 {
                    // both the cones are symmetrical
                    let leftbwidth = left.pos.basewidth_going_east.unwrap();
                    let rightbwidth = right.pos.basewidth_going_east.unwrap();
        
                    slot.pos.basewidth_going_east = Some( leftbwidth + rightbwidth +2 ); // the 2 is the extra padding for the special case
                    slot.pos.basewidth_going_west = Some( leftbwidth + rightbwidth +2 ); // the 2 is the extra padding for the special case
                    slot.pos.tipwidth_going_east = Some(0);
                    slot.pos.tipwidth_going_west = Some(0);
                    slot.pos.h = Some( cmp::max(left.pos.h.unwrap(), right.pos.h.unwrap()) + 2 );
                    slot.pos.branch_type = Some(BranchType::Cone);

                    println!("cone1 bw {} tw {} h {}", slot.pos.basewidth_going_east.unwrap(), slot.pos.tipwidth_going_west.unwrap(), slot.pos.h.unwrap());
                
                    left.pos.y = Some( -2 );
                    right.pos.y = Some( -2 );
                    left.pos.x = Some( -2 );
                    right.pos.x = Some( 2 );
                    return;

                // if one is a small cone:
                } else if left.pos.h.unwrap() == 3 || right.pos.h.unwrap() == 3 {

                    // WARNING: if this special case is taken off, a bug will often appear
                    // where the trapezoid's tip goes too far down and out of the usual space
                    // that means the usual relation between the 2 widths and h is not valid anymore
                    // and that causes the trees to overlap
                    // I've never seen the bug appear with the special case active, but who knows!

                    let leftbwidth = left.pos.basewidth_going_east.unwrap();
                    let rightbwidth = right.pos.basewidth_going_east.unwrap();
        
                    slot.pos.basewidth_going_east = Some( leftbwidth + rightbwidth + 1 );
                    slot.pos.basewidth_going_west = Some( leftbwidth + rightbwidth + 1 );
                    slot.pos.tipwidth_going_east = Some(0);
                    slot.pos.tipwidth_going_west = Some(0);
                    slot.pos.h = Some( cmp::max(left.pos.h.unwrap(), right.pos.h.unwrap()) + 3 );
                    slot.pos.branch_type = Some(BranchType::Cone);

                    println!("cone2 bw {} tw {} h {}", slot.pos.basewidth_going_east.unwrap(), slot.pos.tipwidth_going_west.unwrap(), slot.pos.h.unwrap());


                    left.pos.y = Some( -3 );
                    right.pos.y = Some( -3 );
                    left.pos.x = Some( -3 );
                    right.pos.x = Some( 3 );
                    return;
                }
            } 
            
            // general case: trapezoid => trapezoid
            // TODO think about adding a min size in case the trapezoid gets bugged (as described above)
            
            let tip_width_left = left.pos.tipwidth_going_east.unwrap();
            let tip_width_right = right.pos.tipwidth_going_west.unwrap();
            let hmin = cmp::min(left.pos.h.unwrap(), right.pos.h.unwrap());
            let d_between_centers = tip_width_left+1 + tip_width_right+1 + 2*(hmin-1) + MIN_SPACING;

            // MAYBE +1?
            slot.pos.basewidth_going_west = Some( left.pos.basewidth_going_west.unwrap() + d_between_centers/2 );
            slot.pos.basewidth_going_east = Some( left.pos.basewidth_going_east.unwrap() + d_between_centers/2 );

            slot.pos.h = Some( cmp::max(left.pos.h.unwrap(), right.pos.h.unwrap()) + 4);
            slot.pos.tipwidth_going_east = Some( slot.pos.basewidth_going_east.unwrap() - slot.pos.h.unwrap() +1 ); 
            slot.pos.tipwidth_going_west = Some( slot.pos.basewidth_going_west.unwrap() - slot.pos.h.unwrap() +1 ); 
            slot.pos.branch_type = Some(BranchType::Trapezoid);
            println!("trapezoid bw {} tw {} h {}", slot.pos.basewidth_going_east.unwrap(), slot.pos.tipwidth_going_west.unwrap(), slot.pos.h.unwrap());

            left.pos.y = Some( -4 );
            right.pos.y = Some( -4 );
            left.pos.x = Some( -d_between_centers/2 );
            right.pos.x = Some( d_between_centers/2 );
            

            // OLD ALGORYTHM for cones n stuff
            // let leftbwidth = left.pos.basewidth.unwrap();
            // let rightbwidth = right.pos.basewidth.unwrap();

            // let wstar = cmp::min(leftbwidth, rightbwidth);
            // let astar = (wstar + 1)/2 ;

            // slot.pos.basewidth = Some( leftbwidth + rightbwidth + 1 );
            
            // left.pos.x = Some(0 - astar );
            // right.pos.x = Some(astar );
            // left.pos.y = Some(0 - astar );
            // right.pos.y = Some(0 - astar );


        }

        NodeType::Zero(a) => {
            match &a.zeronode {
                ZeroNodeType::Number(_) => {
                    // for numbers, the noderesult and the width is hard-coded at build time.
                }
            }
        }

        ////////////////////
        NodeType::Many(mslot) => {

            for j in &mut mslot.leaves {
                    if j.pos.h == None {  set_pos(j); }
                }


            match &mslot.manynode {
                // ManyNodeType::ManySum(_) => {
                    
                //     let mut ww = 0;
                //     for j in &mut mslot.leaves {
                //         if j.pos.tipwidth == None {  set_pos(j); }
                //         eprintln!("error here 1");
                //         ww += slot.pos.width.unwrap();
                //         ww += 1;
                //     }
                //     if mslot.leaves.len() != 1 {  ww -= 1;  }

                //     let mut currx = -ww/2;
                //     for j in &mut mslot.leaves {
                //         j.pos.y =  Some( -1 );   
                //         j.pos.x = Some( currx );
                //         currx += j.pos.width.unwrap() + 1;
                //     }
                    
                // }

                ManyNodeType::Times(_) => {
                    

                    println!("baa baa mslot.leaves.len() {}", mslot.leaves.len());
                    if mslot.leaves.len() == 1 { // zero times

                        let lonenode = &mut mslot.leaves[0];
                        lonenode.pos.y =  Some( -3 );
                        lonenode.pos.x = Some( 0 );
                        slot.pos.basewidth_going_west = Some( lonenode.pos.basewidth_going_west.unwrap() );
                        slot.pos.basewidth_going_east = Some( lonenode.pos.basewidth_going_east.unwrap() );
                        slot.pos.tipwidth_going_west = Some( lonenode.pos.tipwidth_going_west.unwrap() -3 );
                        slot.pos.tipwidth_going_east = Some( lonenode.pos.tipwidth_going_east.unwrap() -3 );
                        slot.pos.h = Some( lonenode.pos.h.unwrap() + 3 );
                        println!("0time bw {} tw {} h {}", slot.pos.basewidth_going_east.unwrap(), slot.pos.tipwidth_going_west.unwrap(), slot.pos.h.unwrap());


                        // maybe bug with tip width like for cone2


                    } else if mslot.leaves.len() == 2 { // one time

                        let leaves = &mut mslot.leaves;

                        let tip_width_left = leaves[0].pos.tipwidth_going_east.unwrap();
                        let tip_width_right = leaves[1].pos.tipwidth_going_west.unwrap();
                        let hmin = cmp::min(leaves[0].pos.h.unwrap(), leaves[1].pos.h.unwrap());
                        let d_between_centers = tip_width_left+1 + tip_width_right+1 + 2*(hmin-1) + MIN_SPACING;
            
                        // MAYBE +1?
                        slot.pos.basewidth_going_west = Some( leaves[0].pos.basewidth_going_west.unwrap() + d_between_centers );
                        slot.pos.basewidth_going_east = Some( leaves[1].pos.basewidth_going_east.unwrap() );
                        slot.pos.h = Some( cmp::max(leaves[0].pos.h.unwrap(), leaves[1].pos.h.unwrap()) + 2);
                        slot.pos.tipwidth_going_east = Some( slot.pos.basewidth_going_east.unwrap() - slot.pos.h.unwrap() +1 ); 
                        slot.pos.tipwidth_going_west = Some( slot.pos.basewidth_going_west.unwrap() - slot.pos.h.unwrap() +1 ); 
                        slot.pos.branch_type = Some(BranchType::Trapezoid);
                        println!("1time bw {} tw {} h {}", slot.pos.basewidth_going_east.unwrap(), slot.pos.tipwidth_going_west.unwrap(), slot.pos.h.unwrap());
            
                        leaves[0].pos.y = Some( -2 );
                        leaves[1].pos.y = Some( -2 );
                        leaves[0].pos.x = Some( -d_between_centers );
                        leaves[1].pos.x = Some( 0 );
                        

                    } else { // n times

                        let leaves = &mut mslot.leaves;
                        // for each pair of leaves
                        let tip_width_left_1 = leaves[1].pos.tipwidth_going_east.unwrap();
                        let tip_width_right_1 = leaves[2].pos.tipwidth_going_west.unwrap();
                        let hmin = cmp::min(leaves[1].pos.h.unwrap(), leaves[2].pos.h.unwrap());
                        let d1 = tip_width_left_1+1 + tip_width_right_1+1 + 2*(hmin-1) + MIN_SPACING;
            


                        let n_expr_leaves = leaves.len() - 1;
                        let exprwtot = d1 * (n_expr_leaves as i32) + ((n_expr_leaves as i32 -1 )*MIN_SPACING )  - (d1 -1) ;
                        // Source(s): dude trust me

                        // expr leaves pos
                        let mut xcount = - exprwtot/2;
                        println!("exprwtot {}", exprwtot);
                        println!("exprw {}", d1);
                        for j in 1..(n_expr_leaves +1 ) {
                            
                            leaves[j].pos.x = Some( xcount);
                            leaves[j].pos.y = Some( -4 );
                            println!("xcount {}", xcount);
                            xcount += d1 + MIN_SPACING;
                        }
                        // times leaf pos
                        let tip_width_left_2 = leaves[0].pos.tipwidth_going_east.unwrap();
                        let tip_width_right_2 = leaves[1].pos.tipwidth_going_west.unwrap();
                        let hmin = cmp::min(leaves[0].pos.h.unwrap(), leaves[1].pos.h.unwrap());
                        let d2 = tip_width_left_2 +1 + tip_width_right_2 +1 + 2*(hmin-1) + MIN_SPACING;

                        leaves[0].pos.x = Some( -exprwtot/2 - d2 - MIN_SPACING);
                        leaves[0].pos.y = Some( -4 );
                        
                        // widths
                        slot.pos.h = Some( cmp::max(leaves[0].pos.h.unwrap(), leaves[1].pos.h.unwrap()) + 2);
                       
                        slot.pos.tipwidth_going_east = Some( exprwtot/2 + leaves[1].pos.tipwidth_going_east.unwrap() + 1);
                        slot.pos.basewidth_going_east = Some( exprwtot/2 + leaves[1].pos.basewidth_going_east.unwrap() + 1);

                        slot.pos.tipwidth_going_west = Some( exprwtot/2 + d2 + leaves[0].pos.tipwidth_going_west.unwrap() + 1 );
                        slot.pos.basewidth_going_west = Some( exprwtot/2 + d2 + leaves[0].pos.basewidth_going_west.unwrap() + 1);
                        slot.pos.branch_type = Some(BranchType::TimesN);

                    }
                    

                }
            }

            ////////////////////
        }
        
    }

}



fn draw_2d_vec(startslot: &mut Box<NodeSlot>) -> Vec<Vec<char>> {

    set_pos(startslot);
    eprintln!("set_pos successful");

    //size array
    let mut v = vec![vec![' '; 150]; 35];
    let startx = 75;
    let starty = 35 - 1;

    fn dive(slot: &Box<NodeSlot>, v: &mut Vec<Vec<char>>, lastx: i32, lasty: i32, ) {
        let this_rel_x = slot.pos.x.unwrap();
        let this_rel_y = slot.pos.y.unwrap();

        // let this_w = slot.pos.width.unwrap();
    
        let newx = lastx + this_rel_x;
        let newy = lasty + this_rel_y;

        let mut number = slot.noderesult.unwrap().value;
        if number < 0 {
            v[(newy) as usize][(newx) as usize - 1] =  '-';
            number = -number;
        }

        let offset = (number.to_string().len() / 2) as usize;
        for (i, digit) in number.to_string().chars().enumerate() {
            // println!("{:?}", slot.pos.width);
            // let nchar = char::from_digit(digit as u32, 10u32).unwrap();
            v[(newy) as usize][(newx + (i  as i32)) as usize - offset] =  digit;
        }
        


        match &slot.node {
            NodeType::Binary(a) => {

                let lchar = match a.binode {
                    BinaryNodeType::Addition(_) => '+',
                    BinaryNodeType::Subtraction(_) => '-',
                    BinaryNodeType::Multiplication(_) => '*',
                    BinaryNodeType::Division(_) => '/',
                    BinaryNodeType::DiceRoll(_) => 'd',
                    BinaryNodeType::Hit(_) => 'h',
                };

                let leftleaf_y = a.leftleaf.pos.y.unwrap();
                v[(newy + leftleaf_y) as usize][(newx) as usize] =  lchar;


                match slot.pos.branch_type {
                    Some( BranchType::Cone ) => {
                        for l in 1..( leftleaf_y.abs() ) {
                            v[(newy - l) as usize][(newx - l ) as usize] = '\\';
                            v[(newy - l) as usize][(newx + l ) as usize] = '/';
                        }

                    }

                    Some(BranchType::Trapezoid) => {
                        let leftleaf_x = a.leftleaf.pos.x.unwrap();

                        for j in (-leftleaf_x.abs() + 2)..=(leftleaf_x.abs() - 2) { 
                            v[(leftleaf_y + newy + 2) as usize][(j + newx) as usize] = '‐'; // '—'
                        }
                        v[(leftleaf_y + newy + 1) as usize][( leftleaf_x + newx + 1) as usize] = '\\';
                        v[(leftleaf_y + newy + 1) as usize][( -leftleaf_x + newx - 1) as usize] = '/';
                        v[(newy - 1) as usize][newx  as usize] =  '|';
                    }
                    _ => {}
                }


            },
            NodeType::Zero(_) => { },
            NodeType::Many(mnode) => { 

                let tnodex = mnode.leaves[0].pos.x.unwrap();
                let tnodey = mnode.leaves[0].pos.y.unwrap();
                // v[(tnodey + newy + 1) as usize][(tnodex + newx) as usize] = '|';
                v[(tnodey + newy ) as usize][(tnodex + newx  + 2) as usize] = '-';
                v[(tnodey + newy ) as usize][(tnodex + newx  + 3) as usize] = '>';

                v[(tnodey + newy) as usize][(tnodex + newx - 2) as usize] = 'x';

                
                
                if mnode.leaves.len() >= 3 {

                    let first_node_x = mnode.leaves[1].pos.x.unwrap();
                    let last_node_x = mnode.leaves.last().unwrap().pos.x.unwrap();

                    let westernmost_k = mnode.leaves.len() - 1;

                    for j in (first_node_x + 2)..(last_node_x - 2) { 
                        v[(tnodey + newy + 2) as usize][(j + newx) as usize] = '‐'; // '—'
                    }

                    let spx = mnode.leaves[1].pos.x.unwrap();
                    v[(tnodey + newy + 1) as usize][(spx + newx + 1) as usize] = '\\';
                    let spx2 = mnode.leaves[westernmost_k].pos.x.unwrap();
                    v[(tnodey + newy + 1) as usize][(spx2 + newx - 1) as usize] = '/';

                    for k in 2..(westernmost_k) {
                        let cx = mnode.leaves[k].pos.x.unwrap();
                        v[(tnodey + newy + 1) as usize][(cx + newx) as usize] = '|';
                    }

                    v[(newy - 1) as usize][newx  as usize] =  '|';

                } else if mnode.leaves.len() == 2 {

                    let cx = mnode.leaves[1].pos.x.unwrap();
                    v[(tnodey + newy + 1) as usize][(cx + newx) as usize] = '|';

                    v[(newy - 1) as usize][newx  as usize] =  '|';

                } else if mnode.leaves.len() == 1 {
                    ///////////
                }



            },
        }


        // if lastwasbin == true {
        //     let length_of_arm =  i32::abs(newx - lastx);
        //     if left == true {
        //         for l in 1..(length_of_arm) {
        //             v[(newy + l) as usize][(newx + l ) as usize] = '\\';
        //         }
        //         v[(newy) as usize][(lastx) as usize] =  lastchar;
        //     } else {
        //         for l in 1..(length_of_arm) {
        //             v[(newy + l) as usize][(newx - l ) as usize] = '/';
        //         }
        //     }
        // }

        //recurs dive
        match &slot.node {
            NodeType::Zero(_) => {},
            NodeType::Binary(a) => {
                dive(&a.leftleaf, v, newx, newy);
                dive(&a.rightleaf, v, newx, newy);
            },
            NodeType::Many(b) => {
                for j in &b.leaves {
                    dive(&j, v, newx, newy);
                }
            }
        }
    }

    startslot.pos.x = Some(0);
    startslot.pos.y = Some(0);
    dive(&startslot, &mut v, startx, starty);

    return v;
}

fn print_2d_vec(v: Vec<Vec<char>>) {
    for i in v {
        for j in i {
            print!("{}",j);
        }
        print!("\n");
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
    Many(ManyNodeSlot),
}

struct ZeroNodeSlot {
    zeronode: ZeroNodeType,
}
struct BinaryNodeSlot {
    leftleaf: Box<NodeSlot>,
    rightleaf: Box<NodeSlot>,
    binode: BinaryNodeType, 
}
struct ManyNodeSlot {
    leaves: Vec<Box<NodeSlot>>,
    manynode: ManyNodeType,
}

enum ZeroNodeType {
    Number(NumberNode),
}
enum BinaryNodeType {
    Addition(AdditionNode),
    Subtraction(SubtractionNode),
    Multiplication(MultiplicationNode),
    Division(DivisionNode),
    DiceRoll(DiceRollNode),
    Hit(HitNode),
}
#[allow(dead_code)]
enum ManyNodeType {
    // ManySum(ManySumNode),
    Times(TimesNode),
}

#[allow(dead_code)]
struct NumberNode { value: i32, }

struct AdditionNode { }
struct SubtractionNode { }
struct MultiplicationNode { }
struct DivisionNode { }
struct DiceRollNode { }
struct HitNode { }

// by convention: in TimesNode, the first leaf is the number of times, the rest are summed
struct TimesNode { }
// struct ManySumNode { }

fn build_number(val: i32) -> NodeSlot {
    let innerinner = ZeroNodeType::Number( NumberNode{value: val} );
    let inner = ZeroNodeSlot{ zeronode: innerinner };
    let res = DiceResult{ value: val, crit: Crit::Normal };
    
    let w;
    if val < 99 {
        w = 1;
    } else if val < 9999 {
        w = 3;
    } else {
        w = val.to_string().len() + 2;
    }
    
    let mut tmppos = EMPTYPOS;
    tmppos.tipwidth_going_east = Some(w as i32 - 1);
    tmppos.tipwidth_going_west = Some(w as i32 - 1);
    tmppos.basewidth_going_east = Some(w as i32 - 1);
    tmppos.basewidth_going_west = Some(w as i32 - 1);
    tmppos.h = Some( 1 );
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

#[derive(Debug, Copy, Clone, PartialEq)]
struct DiceResult {
    value: i32,
    crit: Crit,
}

struct NodePositionInfo { 
    x: Option<i32>, // relative to parent!
    y: Option<i32>,
    tipwidth_going_west: Option<i32>, // width of tip of downwards trapezoid 
    tipwidth_going_east: Option<i32>, // width of tip of downwards trapezoid 
    basewidth_going_west: Option<i32>, // width of base of downwards trapezoid
    basewidth_going_east: Option<i32>, // width of base of downwards trapezoid
    h: Option <i32>,
    branch_type: Option<BranchType>,
}
enum BranchType {
    Cone,
    Trapezoid,
    TimesZero,
    TimesOne,
    TimesN,
}

const EMPTYPOS: NodePositionInfo = NodePositionInfo{ x: None, y: None, tipwidth_going_west: None, tipwidth_going_east: None, basewidth_going_west: None, basewidth_going_east:None, h: None, branch_type: None};


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
                } else {
                    panic!("character h not understood, did you mean \"hit\" ?")
                }
            },

            // CHAR -> TOKEN BLOCK
            '(' => { res.push( Token::LeftPar ); },
            ')' => { res.push( Token::RightPar ); },
            '|' => { res.insert(0, Token::LeftPar); res.push( Token::RightPar ); } // TODO: BAD
            'd' => { res.push( Token::DiceRoll ); },
            'x' => { res.push( Token::Times ); },
            '/' => { res.push( Token::Division ); },
            '*' => { res.push( Token::Multiplication ); },
            '+' => { res.push( Token::Addition ); },
            '-' => { res.push( Token::Subtraction ); },

            other => { panic!("character {} not understood", other); },
        }

        n += 1;
    }


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


fn parse_expression(vec: &[Token]) -> Box<NodeSlot> {
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
    // if no splitter is found at zero par balance, we have 3 cases: 
    // a number (base case)
    // just (...), thus we parse recursively the ... without the parentheses
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
            return Box::new( build_number(temp)   );

        } else {
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


        return build_node_from_splitter(vec, splitter, poz);


    }

}




use std::io::{self};

// fn parse(expr: &str) -> Box<NodeSlot> {
//     return parse_expression(clear_spaces(&mut to_tokens(expr)));
// }


fn main()  {

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
        let mut mola = parse_expression(&tokens);
        fill(&mut mola);
        println!("result: {}", mola.noderesult.unwrap().value);

        let arr = draw_2d_vec(&mut mola);
        print_2d_vec(arr);
        
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
}