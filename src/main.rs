

use rustyline::error::ReadlineError;
use rustyline::Editor;



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
fn build_node_from_splitter(vec: &[Token], splitter: &Token, poz: usize, rng: &mut rand::rngs::ThreadRng) -> Result<Box<NodeSlot>, String> {
    
    
    // complex cases
    match splitter {

        Token::DiceRoll => {
            // d6 is short for 1d6 
            let innerinner = BinaryNodeType::DiceRoll(DiceRollNode{});
            let rightside = parse_expression(&vec[poz+1..], rng)?;
            let leftside;
            if poz == 0 {
                leftside = Box::new(  build_number(1)  );
            } else {
                leftside = parse_expression(&vec[0..poz], rng)?;
            }

            let inner = BinaryNodeSlot{ leftleaf: leftside, rightleaf: rightside, binode: innerinner, };
            return Ok(Box::new( NodeSlot{ pos: EMPTYPOS, noderesult: None, node: NodeType::Binary(inner), } ));
        },

        Token::Times => {
            let n_times_string;
            let expr_string;
            // as default, (A) x (B) means "roll A copies of B".
            // except if A is a long expression and B is a single number, like in (4d6 + 5)x3: 
            // in this care it's B copies of A (3 copies of the parenthesis).
            if vec[poz+1..].len() == 1 && vec[0..poz].len() != 1 {
                if let Token::Number(_num) = vec[poz+1] {
                    // n times is on the right
                    // eprintln!("special case with number {} on the right", num);
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

            let mut n_times_leaf = parse_expression(n_times_string, rng)?;
            fill(&mut n_times_leaf, rng);
            let n_times = n_times_leaf.noderesult.unwrap().value;

            let mut temp_v = Vec::with_capacity(n_times as usize + 1);
            temp_v.push(n_times_leaf); 

            for _j in 0..n_times {
                let jleaf = parse_expression(expr_string, rng)?;
                temp_v.push(jleaf);
            }

            let innerinner = ManyNodeType::Times(TimesNode{});
            let inner = ManyNodeSlot{ leaves: temp_v, manynode: innerinner };
            return Ok(Box::new( NodeSlot{ pos: EMPTYPOS, noderesult: None, node: NodeType::Many(inner), } ));


        }

        _ => { },
    }
    

    // simple cases, with the recursive parsing grouped outside of the match
    // it might a bad idea to put the parsing out of the match anyways,
    // because it still get called if the splitter is wrong!
    if vec[0..poz].len() == 0usize {

    }
    let leftside = parse_expression(&vec[0..poz], rng)?;
    let rightside = parse_expression(&vec[poz+1..], rng)?;
    
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
    return Ok(Box::new( NodeSlot{ pos: EMPTYPOS, noderesult: None, node: NodeType::Binary(inner), } ));



}

fn fill(slot: &mut Box<NodeSlot>, rng: &mut rand::rngs::ThreadRng) {
    match &mut slot.node {
        NodeType::Binary(a) => {

            
            let left = &mut a.leftleaf;
            let right = &mut a.rightleaf;

            if left.noderesult == None {  fill(left, rng); }
            if right.noderesult == None {  fill(right, rng);  }

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
                        n += rng.gen_range(1, rightres.value + 1);
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
                        if j.noderesult == None {  fill(j, rng); }
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

                    eprintln!("cone1 bw_east {} tw_east {} bw_west {} tw_west {} h {}", slot.pos.basewidth_going_east.unwrap(), slot.pos.tipwidth_going_east.unwrap(),slot.pos.basewidth_going_west.unwrap(), slot.pos.tipwidth_going_west.unwrap(), slot.pos.h.unwrap());
                
                    left.pos.y = Some( -2 );
                    right.pos.y = Some( -2 );
                    left.pos.x = Some( -2 );
                    right.pos.x = Some( 2 );
                    return;

                // if one is a small cone:
                } else if left.pos.h.unwrap() == 3 || right.pos.h.unwrap() == 3 {

                    // WARNING: IF this special case is taken off, a bug will often appear
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

                    eprintln!("cone2 bw_east {} tw_east {} bw_west {} tw_west {} h {}", slot.pos.basewidth_going_east.unwrap(), slot.pos.tipwidth_going_east.unwrap(),slot.pos.basewidth_going_west.unwrap(), slot.pos.tipwidth_going_west.unwrap(), slot.pos.h.unwrap());


                    left.pos.y = Some( -3 );
                    right.pos.y = Some( -3 );
                    left.pos.x = Some( -3 );
                    right.pos.x = Some( 3 );
                    return;
                }
            } 
            
            // general case: trapezoid => trapezoid
            // TODO think about adding a min size in case the trapezoid gets bugged (as described above)
            
            // let tip_width_left = left.pos.tipwidth_going_east.unwrap();
            // let tip_width_right = right.pos.tipwidth_going_west.unwrap();
            let mut tip_width_left = left.pos.tipwidth_going_east.unwrap();
            if tip_width_left < 1  {
                tip_width_left = 1 ;
            }
            
            let mut tip_width_right = right.pos.tipwidth_going_west.unwrap();
            if tip_width_right < 1  {
                tip_width_right = 1 ;
            }

            let hmin = cmp::min(left.pos.h.unwrap(), right.pos.h.unwrap());
            let d_between_centers = tip_width_left+1 + tip_width_right+1 + 2*(hmin-1) + MIN_SPACING;

            slot.pos.basewidth_going_west = Some( left.pos.basewidth_going_west.unwrap() + d_between_centers/2 );
            slot.pos.basewidth_going_east = Some( left.pos.basewidth_going_east.unwrap() + d_between_centers/2 );

            slot.pos.h = Some( cmp::max(left.pos.h.unwrap(), right.pos.h.unwrap()) + 4);
            slot.pos.tipwidth_going_east = Some( slot.pos.basewidth_going_east.unwrap() - slot.pos.h.unwrap() +1 );
            if slot.pos.tipwidth_going_east < Some(0) { slot.pos.tipwidth_going_east = Some(0)};

            slot.pos.tipwidth_going_west = Some( slot.pos.basewidth_going_west.unwrap() - slot.pos.h.unwrap() +1 );
            if slot.pos.tipwidth_going_west < Some(0) { slot.pos.tipwidth_going_east = Some(0)};
             
            slot.pos.branch_type = Some(BranchType::Trapezoid);
            eprintln!("trapezoid bw_east {} tw_east {} bw_west {} tw_west {} h {}", slot.pos.basewidth_going_east.unwrap(), slot.pos.tipwidth_going_east.unwrap(),slot.pos.basewidth_going_west.unwrap(), slot.pos.tipwidth_going_west.unwrap(), slot.pos.h.unwrap());

            left.pos.y = Some( -4 );
            right.pos.y = Some( -4 );
            left.pos.x = Some( -d_between_centers/2 );
            right.pos.x = Some( d_between_centers/2 );
            


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
                    

                    // eprintln!("baa baa mslot.leaves.len() {}", mslot.leaves.len());
                    if mslot.leaves.len() == 1 { // zero times

                        let lonenode = &mut mslot.leaves[0];
                        lonenode.pos.y =  Some( -3 );
                        lonenode.pos.x = Some( 0 );
                        slot.pos.basewidth_going_west = Some( lonenode.pos.basewidth_going_west.unwrap() );
                        slot.pos.basewidth_going_east = Some( lonenode.pos.basewidth_going_east.unwrap() );
                        slot.pos.tipwidth_going_west = Some( lonenode.pos.tipwidth_going_west.unwrap() -3 );
                        if slot.pos.tipwidth_going_west < Some(0) { slot.pos.tipwidth_going_west = Some(0)};
                        slot.pos.tipwidth_going_east = Some( lonenode.pos.tipwidth_going_east.unwrap() -3 );
                        if slot.pos.tipwidth_going_east < Some(0) { slot.pos.tipwidth_going_east = Some(0)};

                        slot.pos.h = Some( lonenode.pos.h.unwrap() + 3 );
                        eprintln!("0time bw_east {} tw_east {} bw_west {} tw_west {} h {}", slot.pos.basewidth_going_east.unwrap(), slot.pos.tipwidth_going_east.unwrap(),slot.pos.basewidth_going_west.unwrap(), slot.pos.tipwidth_going_west.unwrap(), slot.pos.h.unwrap());


                        // maybe bug with tip width like for cone2


                    } else if mslot.leaves.len() == 2 { // one time

                        let leaves = &mut mslot.leaves;

                        let tip_width_left = leaves[0].pos.tipwidth_going_west.unwrap();
                        let tip_width_right = leaves[1].pos.tipwidth_going_east.unwrap();

                        eprintln!("twleft {}, twright {}", tip_width_left, tip_width_right);

                        let hmin = cmp::min(leaves[0].pos.h.unwrap(), leaves[1].pos.h.unwrap());
                        let d_between_centers = tip_width_left+1 + tip_width_right+1 + 2*(hmin-1) + MIN_SPACING +1;
                        // the +1 is added for aesthetic reasons with little understanding of what's going on!
                        eprintln!("d_b_centers {}", d_between_centers);
            
                        slot.pos.basewidth_going_west = Some( leaves[0].pos.basewidth_going_west.unwrap() + d_between_centers );
                        slot.pos.basewidth_going_east = Some( leaves[1].pos.basewidth_going_east.unwrap() );

                        slot.pos.h = Some( cmp::max(leaves[0].pos.h.unwrap(), leaves[1].pos.h.unwrap()) + 2);

                        slot.pos.tipwidth_going_east = Some( slot.pos.basewidth_going_east.unwrap() - slot.pos.h.unwrap() +1 );
                        if slot.pos.tipwidth_going_east < Some(0) { slot.pos.tipwidth_going_east = Some(0)};
                        slot.pos.tipwidth_going_west = Some( slot.pos.basewidth_going_west.unwrap() - slot.pos.h.unwrap() +1 +2);
                        if slot.pos.tipwidth_going_west < Some(0) { slot.pos.tipwidth_going_west = Some(0)};

                        slot.pos.branch_type = Some(BranchType::Trapezoid);
                        eprintln!("1time bw_east {} tw_east {} bw_west {} tw_west {} h {}", slot.pos.basewidth_going_east.unwrap(), slot.pos.tipwidth_going_east.unwrap(),slot.pos.basewidth_going_west.unwrap(), slot.pos.tipwidth_going_west.unwrap(), slot.pos.h.unwrap());
            
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
                        eprintln!("exprwtot {}", exprwtot);
                        for j in 1..(n_expr_leaves +1 ) {
                            
                            leaves[j].pos.x = Some( xcount);
                            leaves[j].pos.y = Some( -4 );
                            // eprintln!("xcount {}", xcount);
                            xcount += d1 + MIN_SPACING;
                        }
                        // times leaf pos
                        let tip_width_left_2 = leaves[0].pos.tipwidth_going_east.unwrap();
                        let tip_width_right_2 = leaves[1].pos.tipwidth_going_west.unwrap();
                        let hmin = cmp::min(leaves[0].pos.h.unwrap(), leaves[1].pos.h.unwrap());
                        let d2 = tip_width_left_2 +1 + tip_width_right_2 +1 + 2*(hmin-1) + MIN_SPACING;
                        
                        leaves[0].pos.x = Some( -exprwtot/2 - d2 - MIN_SPACING);
                        leaves[0].pos.y = Some( -4 );
                        
                        // eprintln!("d2 {}", d2);
                        // widths
                        slot.pos.h = Some( cmp::max(leaves[0].pos.h.unwrap(), leaves[1].pos.h.unwrap()) + 4);
                        
                        slot.pos.tipwidth_going_east = Some( exprwtot/2 + leaves[1].pos.tipwidth_going_east.unwrap() -4);
                        if slot.pos.tipwidth_going_east < Some(0) { slot.pos.tipwidth_going_east = Some(0)};

                        // the +1 going west is because of the x
                        slot.pos.tipwidth_going_west = Some( exprwtot/2 + d2 + leaves[0].pos.tipwidth_going_west.unwrap() -4 +1);
                        if slot.pos.tipwidth_going_west < Some(0) { slot.pos.tipwidth_going_west = Some(0)};
                        
                        slot.pos.basewidth_going_east = Some( exprwtot/2 + leaves[1].pos.basewidth_going_east.unwrap());
                        slot.pos.basewidth_going_west = Some( exprwtot/2 + d2 + leaves[0].pos.basewidth_going_west.unwrap() +1);
                        // (this one is a bit off but should be ok)

                        slot.pos.branch_type = Some(BranchType::TimesN);

                        eprintln!("Ntimes bw_east {} tw_east {} bw_west {} tw_west {} h {}", slot.pos.basewidth_going_east.unwrap(), slot.pos.tipwidth_going_east.unwrap(),slot.pos.basewidth_going_west.unwrap(), slot.pos.tipwidth_going_west.unwrap(), slot.pos.h.unwrap());


                    }
                    

                }
            }

            ////////////////////
        }
        
    }

}

struct MaxVals {
    max_x: i32,
    min_x: i32,
    min_y: i32,
}

// overwrite x and y as absolute pos and returns max_x min_x min_y
// has to be called right after setpos
fn set_pos_abs( mut startslot: &mut Box<NodeSlot> ) -> MaxVals {

    let mut max = MaxVals{ max_x: startslot.pos.x.unwrap(), 
        min_x: startslot.pos.x.unwrap(),
        min_y: startslot.pos.y.unwrap(), };

    let startx = 0;
    let starty = 0;
    startslot.pos.x = Some(0);
    startslot.pos.y = Some(0);
    dive2(&mut startslot, startx, starty, &mut max);

    fn dive2(slot: &mut Box<NodeSlot>, lastx: i32, lasty: i32, max: &mut MaxVals) {

        let this_rel_x = slot.pos.x.unwrap();
        let this_rel_y = slot.pos.y.unwrap();
    
        let newx = lastx + this_rel_x;
        let newy = lasty + this_rel_y;

        if newx > max.max_x { max.max_x = newx; } 
        else if newx < max.min_x { max.min_x = newx; }

        if newy < max.min_y { max.min_y = newy; } 

        slot.pos.x = Some( newx );
        // eprintln!("newx {} ", newx);
        slot.pos.y = Some( newy );
        // eprintln!("newy {} ", newy);

        //recurs dive
        match &mut slot.node {
            NodeType::Zero(_) => {},
            NodeType::Binary(a) => {
                
                dive2(&mut a.leftleaf, newx, newy, max);
                dive2(&mut a.rightleaf, newx, newy, max);
            },
            NodeType::Many(b) => {
                for mut j in &mut b.leaves {
                    dive2(&mut j, newx, newy, max);
                }
            }
        }
    }

    return max;

} 


// traslate everything by fix amount on x and y
fn trasl( mut slot: &mut Box<NodeSlot>, xtrasl: i32, ytrasl: i32  ) {

    let oldx = slot.pos.x.unwrap();
    slot.pos.x = Some( oldx + xtrasl );
    // eprintln!("traslated x {} ", oldx + xtrasl );
    let oldy = slot.pos.y.unwrap();
    slot.pos.y = Some( oldy +    ytrasl );
    // eprintln!("traslated y {} ", oldy + ytrasl );

    //recurs
    match &mut slot.node {
        NodeType::Zero(_) => {},
        NodeType::Binary(a) => {
            
            trasl(&mut a.leftleaf, xtrasl, ytrasl);
            trasl(&mut a.rightleaf, xtrasl, ytrasl);
        },
        NodeType::Many(b) => {
            for mut j in &mut b.leaves {
                trasl(&mut j, xtrasl, ytrasl);
            }
        }
    }
}




fn draw_2d_vec(startslot: &mut Box<NodeSlot>) -> Vec<Vec<char>> {

    set_pos(startslot);
    startslot.pos.x = Some(0);
    startslot.pos.y = Some(0);

    let max = set_pos_abs(startslot);
    // println!("max x {}", max.max_x);
    // println!("min x {}", max.min_x);
    // println!("max y {}", max.min_y);

    let full_w = (max.max_x - max.min_x) +1 ;

    const XLEFTPAD: usize = 4;
    const XRIGHTPAD: usize = 4;
    const YUPPAD: usize = 1;
    const YDOWNPAD: usize = 1;
    
    let vxsize = full_w as usize + XLEFTPAD + XRIGHTPAD;
    let vysize = max.min_y.abs() as usize + YUPPAD + YDOWNPAD + 1; 
    let mut v = vec![vec![' '; vxsize +1 ]; vysize];
    // eprintln!("vxsize {}", vxsize);
    // eprintln!("vysize {}", vysize);

    trasl(startslot, -max.min_x  + XLEFTPAD as i32 , -max.min_y + YUPPAD as i32 );

    write(&startslot, &mut v);
    return v;



    fn write(slot: &Box<NodeSlot>, v: &mut Vec<Vec<char>>, ) {  
        let newx = slot.pos.x.unwrap();
        let newy = slot.pos.y.unwrap();

        // eprintln!("writing x {} ", newx);
        // eprintln!("writing y {} ", newy);

        let mut number = slot.noderesult.unwrap().value;
        if number < 0 {
            let mut stepback = 1;
            if number < -99999 {  stepback = number.to_string().len() / 2 +1; }
            else if number < -999 { stepback = 4;}
            else if number < -9 { stepback = 2;}


            v[(newy) as usize][(newx) as usize - stepback] =  '-';
            // eprintln!("minus x {} ", newx -1);
            // eprintln!("minus y {} ", newy);

            number = -number;
        }

        let offset = (number.to_string().len() / 2) as usize;
        for (i, digit) in number.to_string().chars().enumerate() {
            v[(newy) as usize][(newx + (i  as i32)) as usize - offset] =  digit;
            // eprintln!("digits x {} ", (newx + (i  as i32)) as usize - offset);
            // eprintln!("digits y {} ", newy);
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
                // eprintln!("lchar x {} ", newx);
                // eprintln!("lchar y {} ", leftleaf_y);
                v[(leftleaf_y) as usize][(newx) as usize] =  lchar;


                match slot.pos.branch_type {
                    
                    Some( BranchType::Cone ) => {
                        let templen = (newy - leftleaf_y).abs();
                        for l in 1..( templen ) {

                            // eprintln!("\\ x {} ", newx - l );
                            // eprintln!("/ y {} ", newy - l);
                            // eprintln!("\\ x {} ", newy - l );
                            // eprintln!("/ y {} ", newx + l);
                            v[(newy - l) as usize][(newx - l ) as usize] = '\\';
                            v[(newy - l) as usize][(newx + l ) as usize] = '/';
                        }

                    }

                    Some(BranchType::Trapezoid) => {
                        let leftleaf_x = a.leftleaf.pos.x.unwrap();
                        let rightleaf_x = a.rightleaf.pos.x.unwrap();
                        // eprintln!("tr x {} ", leftleaf_x );


                        for j in (leftleaf_x.abs() + 2)..=(rightleaf_x.abs() - 2) { 
                            v[(leftleaf_y + 2) as usize][(j) as usize] = '‐'; // '—'
                        }
                        v[(leftleaf_y + 1) as usize][( leftleaf_x + 1) as usize] = '\\';
                        v[(leftleaf_y + 1) as usize][( rightleaf_x - 1) as usize] = '/';
                        v[(newy - 1) as usize][newx  as usize] =  '|';
                    }
                    _ => {}
                }


            },
            NodeType::Zero(_) => { },
            NodeType::Many(mnode) => { 

                let tnodex = mnode.leaves[0].pos.x.unwrap();
                let tnodey = mnode.leaves[0].pos.y.unwrap();

                // arrow
                if mnode.leaves.len() == 1 { // zero times
                    v[( tnodey + 1 ) as usize][( tnodex ) as usize] = '|';
                    v[( tnodey + 2 ) as usize][( tnodex ) as usize] = '|';

                    v[( tnodey ) as usize][( tnodex  + 2+1) as usize] = '.';
                    v[( tnodey ) as usize][( tnodex  + 3+1) as usize] = '.';
                    v[( tnodey ) as usize][( tnodex  + 4+1) as usize] = '.';
                } else {

                    let first_node_x = mnode.leaves[1].pos.x.unwrap();
                    for j in (tnodex+2)..=(first_node_x-3) {
                        v[( tnodey ) as usize][( j) as usize] = '-';
                    }
                    v[( tnodey ) as usize][( first_node_x -2 ) as usize] = '>';
                }

                v[( tnodey) as usize][( tnodex - 1) as usize] = 'x';

                
                // scaffolding
                if mnode.leaves.len() >= 3 { // 2 or more times

                    let first_node_x = mnode.leaves[1].pos.x.unwrap();
                    let last_node_x = mnode.leaves.last().unwrap().pos.x.unwrap();

                    let westernmost_k = mnode.leaves.len() - 1;

                    for j in (first_node_x + 2)..=(last_node_x - 2) { 
                        v[(tnodey + 2) as usize][(j) as usize] = '‐'; // '—'
                    }

                    let spx = mnode.leaves[1].pos.x.unwrap();
                    v[(tnodey + 1) as usize][(spx + 1) as usize] = '\\';
                    let spx2 = mnode.leaves[westernmost_k].pos.x.unwrap();
                    v[(tnodey + 1) as usize][(spx2 - 1) as usize] = '/';

                    for k in 2..(westernmost_k) {
                        let cx = mnode.leaves[k].pos.x.unwrap();
                        v[(tnodey + 1) as usize][(cx) as usize] = '|';
                    }
                    
                    let plus_spacing =  mnode.leaves[2].pos.x.unwrap() - mnode.leaves[1].pos.x.unwrap();

                    for k in 1..(westernmost_k) {
                        let cx = mnode.leaves[k].pos.x.unwrap();
                        v[(tnodey ) as usize][(cx + plus_spacing/2) as usize] = '+';
                    }

                    v[(newy - 1) as usize][newx  as usize] =  '|';

                } else if mnode.leaves.len() == 2 { // one time

                    let cx = mnode.leaves[1].pos.x.unwrap();
                    v[(tnodey + 1) as usize][(cx) as usize] = '|';

                    v[(newy - 1) as usize][newx  as usize] =  '|';

                } else if mnode.leaves.len() == 1 {
                    ///////////
                }



            },
        }


        //recurs write
        match &slot.node {
            NodeType::Zero(_) => {},
            NodeType::Binary(a) => {
                write(&a.leftleaf, v, );
                write(&a.rightleaf, v, );
            },
            NodeType::Many(b) => {
                for j in &b.leaves {
                    write(&j, v, );
                }
            }
        }
    }


}


fn print_2d_vec(v: Vec<Vec<char>>) {
    
    let termw = get_term_width();
    // eprintln!("termw {}", termw);
    let xlen = v[0].len();
    // eprintln!("xlen {}", xlen);
    // eprintln!("xlen/termw {}", xlen/termw);

    if xlen >= termw {
        for n in 0..(xlen/termw){


            for i in 0..v.len() {
                for j in n*termw..(n+1)*termw {
                    print!("{}",v[i][j]);
                }
                print!("\n");
            }
        }

    }


    for i in 0..v.len() {
        for j in (xlen/termw)*termw..(xlen) {
            // println!("i {} j {}", i,j);
            print!("{}",v[i][j]);
        }
        print!("\n");
    }
   
}

fn get_term_width() -> usize {
    let term_width;
    if let Some((w, _h)) = term_size::dimensions() {
        term_width = w;
    } else {
    println!("Unable to get term size :(");
    term_width = 80;
    }
    return term_width;
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
        w = val.to_string().len() / 2 +1;
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
    // TimesZero,
    // TimesOne,
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

fn to_tokens(expr: &str) -> Result<Vec<Token>, String> { 
    
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
                    return Err("character h not understood, did you mean \"hit\" ?".to_string());
                    // panic!("character h not understood, did you mean \"hit\" ?")
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

            other => { 
                let mut exit_str = "character ".to_string();
                exit_str.push(other);
                exit_str.push_str(" not understood");
                return Err(exit_str); 
            },
            // other => { panic!("character {} not understood", other); },
        }

        n += 1;
    }


    return Ok(res);
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


fn parse_expression(vec: &[Token], rng: &mut rand::rngs::ThreadRng) -> Result<Box<NodeSlot>, String> {
    // eprintln!("expr {:?}", vec);
    match vec.len() {
        0 => {
            let exit_str = format!("string not understood");
            return Err(exit_str);
        },
        1 => {
            if let Token::Number(i) = vec[0] {
                return Ok(Box::new( build_number(i)   ));
            }else{
                let exit_str = format!("string not understood: found {:?} instead of a number", vec[0]);
                return Err(exit_str);
            }
        },
        _n => {

            let mut par_balance: i8 = 0;
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

            if let None = position_of_splitter {
        
                if let Token::LeftPar = vec[0] { 
                    // eprintln!("no splitter found, got to clear parentheses maybe"); //PARSING DEBUG
                    return parse_expression(&vec[1..vec.len()-1], rng);
                } else {
                    let exit_str = format!("sequence {:?} not understood", vec);
                    return Err(exit_str);
                }
        
            } else {
                let poz = position_of_splitter.unwrap();
                let splitter = &vec[poz];
        
        
                return Ok( build_node_from_splitter(vec, splitter, poz, rng)?  );
        
        
            }


        }
    }



    // // figure out if the expression is of the form (...)+(...),
    // // where (...) can also have no parentheses but a lower priority op, like in 4*5+4*5
    // // the case (...)+(...)+(...) works, but with two levels
    // // eprintln!("parsing {:?}", vec); //PARSING DEBUG
    // let mut par_balance: u8 = 0;
    // let mut position_of_splitter: Option<usize> = None;
    // 'outer: for spl in &SPLITTERS{
    //     for (n, x) in vec.iter().enumerate() {        

    //         match x {
    //             Token::LeftPar => par_balance += 1,
    //             Token::RightPar => par_balance -= 1,
    //             other if other == spl => {

    //                 if par_balance == 0 {

    //                     position_of_splitter = Some(n);
    //                     break 'outer;
    //                 }
    //             }
    //             _ => {},              
            
    //         }
    //     }
    // }
    // // if no splitter is found at zero par balance, we have 3 cases: 
    // // a number (base case)
    // // just (...), thus we parse recursively the ... without the parentheses
    // // an expression without parentheses

    // if let None = position_of_splitter {
        
    //     // single number case
    //     if vec.len() == 1 {
    //         // eprintln!("no splitter found, single number!"); //PARSING DEBUG
    //         let temp;
    //         if let Token::Number(i) = vec[0] {
    //             temp = i;
    //         }else{
    //             panic!("lol");
    //         }
    //         return Ok(Box::new( build_number(temp)   ));

    //     } else if let Token::LeftPar = vec[0] { 
    //         // eprintln!("no splitter found, got to clear parentheses maybe"); //PARSING DEBUG
    //         return parse_expression(&vec[1..vec.len()-1]);
    //     } else {
    //         return Err("malformed string".to_string());
    //     }

        

    // } else {
    //     let poz = position_of_splitter.unwrap();
    //     let splitter = &vec[poz];


    //     return Ok(build_node_from_splitter(vec, splitter, poz));


    // }

}




fn parsedice(dice_expr :&Vec<Token>, rng: &mut rand::rngs::ThreadRng) -> Result<DiceResult, String> {
    match &mut parse_expression(dice_expr, rng) {
        Ok(a) => {
            fill(a, rng);
            //println!("result: {}", a.noderesult.unwrap().value);
        
            let arr = draw_2d_vec(a);
            print_2d_vec(arr);
        
            println!("result: {}", a.noderesult.unwrap().value);
            return Ok(a.noderesult.unwrap());
        },

        Err(err) => {
            println!("{}", err);
            return Err(err.to_string());
        }


    }


}

// fn save_history() -> Result<(), String>{
     

//     let mut path = dirs::data_local_dir().ok_or("error")?;

    
//     match &mut dirs::data_local_dir() {
//         Some(path) => {
//             path.push("/.roll");
//             match &mut std::fs::create_dir_all(path){
//                 Ok(p2) => {
//                     p2.push("/history.txt");
//                     rl.save_history(  p2 ).unwrap();
                    
//                 },
//                 Err(_) => {
//                     println!("cannot find the right directory, printing history here");
//                     rl.save_history("history.txt").unwrap();
//                 },
//             }


//         },
//         None => {
//             println!("cannot find the right directory, printing history here");
//             rl.save_history("history.txt").unwrap();
//         },
//     }
// }

fn main()  {

    let mut rng = rand::thread_rng();

    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {

                if line == "exit" || line == "Exit" || line == "quit" || line == "Quit" || line == "q" {
                    break
                }

                rl.add_history_entry(line.as_str());

                let expression = line.trim();
                let mut restokens = to_tokens(&expression);
                match &mut restokens {
                    Err(a) => println!("{}",a),
                    Ok(b) => {
                        let tokens = clear_spaces(b);

                        match parsedice(tokens, &mut rng) {
                            Err(_err) => {
                                println!("parsing error: {:?}", _err);
                            },
                            Ok(_b) => {},
                        }

                    },
                }


            },
            Err(ReadlineError::Interrupted) => {
                println!("Interrupt");
            },
            Err(ReadlineError::Eof) => {
                println!("Exit");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
   
    
    // FREQUENCY TEST
    
    // let big_number = 100000;
    // let expression = "1d20";  // MAKE THE DICE SIZE MATCH THE VEC! LEN
    // let mut counter = vec![0;20];
    // for _i in 0..big_number {

    //         let expression = expression.trim();
    //         let mut restokens = to_tokens(&expression);
    //         let mut res = 0;
    //         match &mut restokens {
    //             Err(a) => println!("{}",a),
    //             Ok(b) => {
    //                 let tokens = clear_spaces(b);
    
    //                 match parsedice(tokens, &mut rng) {
    //                     Ok(a) => res = a.value,
    //                     Err(_) => res = 0,

    //                 } 


    //             },
    //         }


    //     counter[res as usize -1] += 1;
    // }

    // let expected = big_number/counter.len();
    // println!("expected: {}  times for each value", expected);
    // for i in 0..counter.len() {
    //     println!("value {} found  {} times, ", i+1, counter[i] )
    // }
}