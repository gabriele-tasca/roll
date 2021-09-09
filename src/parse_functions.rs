use std::char;
use std::cmp;



use rand_xoshiro::rand_core::RngCore;
use rand_xoshiro::rand_core::SeedableRng;

// In this program, all subtyping is done via SUM TYPES (enums, enums in structs, ...).
// It's a pretty good solution formally, but the Rust syntax for enums and structs can get kind of verbose.
// The other (possibly more common) solution would be inheritance, but Rust doesn't implement it.
// Traits work in a very similar way to inheritance, but they lack some features, especially the ability 
// to have fields in traits. Technically you can work around this 
// with get and set functions for each field you'd want in the parent class,
// but those would need to have be implemented identically a bunch of times in each derived class. 

// To implement a new function: 
// 1) Add a new value in the Token enum
// 2) Add the new Token in the priority array, and adjust the length in the type signature (or make it not const once and for all)
// 3) Add the parsing rule in the CHAR -> TOKEN BLOCK
// 4) Add the new Node struct in the TYPE THEORY block
// 5) Add a rule for building the node in the build_node_from_splitter function
// 6) Add a eval rule in the fill function
// 7) Update the draw function. Usually this means both set_pos and the ascii part (draw_2d_vec)


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

// priority array:
// the order here defines the execution priority 
const SPLITTERS: [Token; 7] = [
    Token::Hit,
    
    Token::Addition, 
    Token::Subtraction,
    
    Token::Division,
    Token::Multiplication,
    
    // ...
    Token::Times,
    Token::DiceRoll,
];


// from a string of tokens in which "splitter" is the one with top priority,
// a node is built for the splitter token. its leaves will be built afterwards, by recursively parsing the 
// remaining parts of the string, left and right of the splitter.
fn build_node_from_splitter(vec: &[Token], splitter: &Token, poz: usize, rng: &mut rand_xoshiro::Xoroshiro128StarStar) -> Result<Box<NodeSlot>, String> {
    
    
    // complex cases
    match splitter {

        // Token::DiceRoll => {
        //     // d6 is short for 1d6 
        //     let innerinner = BinaryNodeType::DiceRoll(DiceRollNode{});
        //     let rightside = build_tree(&vec[poz+1..], rng)?;
        //     let leftside;
        //     if poz == 0 {
        //         leftside = Box::new(  build_number(1)  );
        //     } else {
        //         leftside = build_tree(&vec[0..poz], rng)?;
        //     }

        //     let inner = BinaryNodeSlot{ leftleaf: leftside, rightleaf: rightside, binode: innerinner, };
        //     return Ok(Box::new( NodeSlot{ pos: EMPTYPOS, noderesult: None, node: NodeType::Binary(inner), } ));
        // },

        Token::DiceRoll => {

            // simplest case: 1dX or just dX, gives the old little cone
            if poz == 0 || poz == 1 {
                match &vec[0]{
                    Token::Number(1) | Token::DiceRoll => {
                        // d6 is short for 1d6 
                        let innerinner = BinaryNodeType::DiceRoll(DiceRollNode{ hidden: false });
                        let rightside = build_tree(&vec[poz+1..], rng)?;
                        let leftside = Box::new(  build_number(1)  );
            
                        let inner = BinaryNodeSlot{ leftleaf: leftside, rightleaf: rightside, binode: innerinner, };
                        return Ok(Box::new( NodeSlot{ pos: EMPTYPOS, noderesult: None, node: NodeType::Binary(inner), } ));
                    }
                    _ => {}
                }    
            } 


            // with times ("dice_special" case).
            // inside here, the diceroll cones are suppressed if the dice size is a single static number 
            // (ex. just 5d6 instead of 5d( 1d20)) 
            
            let n_times_string = &vec[0..poz];
            let expr_string = &vec[poz+1..];
            
            let must_suppress_cones = {expr_string.len() == 1};

            let mut n_times_leaf = build_tree(n_times_string, rng)?;
            fill(&mut n_times_leaf, rng);
            // let n_times = n_times_leaf.noderesult.unwrap().value;
            let n_times = cmp::max(0,n_times_leaf.noderesult.unwrap().value);


            let mut temp_v = Vec::with_capacity(n_times as usize + 1);
            temp_v.push(n_times_leaf);


            // build n diceroll nodes with just 1 dice each 
            // the dice sizes are all parsed separately, which is probably useless, but cool.
            
            let mut single_dice_size: Option<i32> = None;

            for j in 0..n_times {

                let dice_innerinner;
                if must_suppress_cones {
                    dice_innerinner = BinaryNodeType::DiceRoll(DiceRollNode{ hidden: true });
                } else {
                    dice_innerinner = BinaryNodeType::DiceRoll(DiceRollNode{ hidden: false });
                }

                let leftside = Box::new(  build_number(1)  );
                let rightside = build_tree(expr_string, rng)?;

                if j == 1 {
                    if must_suppress_cones {
                        single_dice_size = Some( rightside.noderesult.unwrap().value );
                    }
                } 

                let dice_inner = BinaryNodeSlot{ leftleaf: leftside, rightleaf: rightside, binode: dice_innerinner, };
                let mut jleaf = Box::new( NodeSlot{ pos: EMPTYPOS, noderesult: None, node: NodeType::Binary(dice_inner), } );
                fill(&mut jleaf, rng);

                temp_v.push(jleaf);
            }



            let times_innerinner = ManyNodeType::Times(TimesNode{ dice_special: single_dice_size});


            let times_inner = ManyNodeSlot{ leaves: temp_v, manynode: times_innerinner };
            return Ok(Box::new( NodeSlot{ pos: EMPTYPOS, noderesult: None, node: NodeType::Many(times_inner), } ));
            /////


        },


        Token::Times => {
            let n_times_string;
            let expr_string;
            // as default, (A) x (B) means "roll A copies of B". (ex. 3x(2d6)), 3 times 2d6
            // except if A is a long expression and B is a single number, like in (4d6 + 5)x3: 
            // in this care it's B copies of A (3 copies of the parenthesis).
            if vec[poz+1..].len() == 1 && vec[0..poz].len() != 1 {
                if let Token::Number(_num) = vec[poz+1] {
                    // n times is on the right
                    // eprintln!("special case with number {} on the right", _num );
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

            let mut n_times_leaf = build_tree(n_times_string, rng)?;
            fill(&mut n_times_leaf, rng);
            // let n_times = n_times_leaf.noderesult.unwrap().value;
            let n_times = cmp::max(0,n_times_leaf.noderesult.unwrap().value);

            let mut temp_v = Vec::with_capacity(n_times as usize + 1);
            temp_v.push(n_times_leaf); 

            for _j in 0..n_times {
                let jleaf = build_tree(expr_string, rng)?;
                temp_v.push(jleaf);
            }

            let innerinner = ManyNodeType::Times(TimesNode{ dice_special: None });
            let inner = ManyNodeSlot{ leaves: temp_v, manynode: innerinner };
            return Ok(Box::new( NodeSlot{ pos: EMPTYPOS, noderesult: None, node: NodeType::Many(inner), } ));


        }

        _ => { }, 
    }
    

    // simple cases, with the recursive parsing grouped outside of the match
    // it might a bad idea to put the parsing out of the match anyways,
    // because it still gets called if the splitter is wrong!
    if vec[0..poz].len() == 0usize {

    }
    let leftside = build_tree(&vec[0..poz], rng)?;
    let rightside = build_tree(&vec[poz+1..], rng)?;
    
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

// take the tree and fill in values at each node, by rolling the rng dice and 
// performing the calculations.
fn fill(slot: &mut Box<NodeSlot>, rng: &mut rand_xoshiro::Xoroshiro128StarStar) {
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
                    // all dicerolls should be just 1 die now. 

                    // let mut n = 0;
                    // // let imax = leftres.value;
                    // let imax = cmp::max(0, leftres.value);
                    // for _i in 0..imax {
                    //     let random_u32 = rng.next_u32();
                    //     let res = dice_from_u32(rightres.value, random_u32);
                    //     n += res;
                    // }
                    // let crit;
                    // if rightres.value == 20 && n == 20 {
                    //     crit = Crit::NatMax;
                    // } else if n == 1 {
                    //     crit = Crit::NatMin;
                    // } else {
                    //     crit = Crit::Normal;
                    // }
                    
                    // let imax = leftres.value;
                    if leftres.value != 1 { panic!("lol")};
                    let random_u32 = rng.next_u32();
                    let n = dice_from_u32(rightres.value, random_u32);
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

// calculate each node's position relative to its root, according to a scheme meant for an 
// ascii representation of the roll tree.
// from each root, set each leaf's position relative to the root, based on the leaves' width.
fn set_pos(slot: &mut Box<NodeSlot>) {
    const MIN_SPACING: i32 = 1;

    match &mut slot.node {

        NodeType::Binary(a) => {
            
            match &a.binode {
                BinaryNodeType::DiceRoll(DiceRollNode{ hidden: true, .. }) => {
                        
                    let left = &mut a.leftleaf;
                    let right = &mut a.rightleaf;

                    if left.pos.h == None {  set_pos(left); }
                    if right.pos.h == None {  set_pos(right);  }

                    let rightbwidth = right.pos.basewidth_going_east.unwrap();
        
                    slot.pos.basewidth_going_east = Some( rightbwidth );
                    slot.pos.basewidth_going_west = Some( rightbwidth );
                    slot.pos.tipwidth_going_east = Some( rightbwidth);
                    slot.pos.tipwidth_going_west = Some( rightbwidth);
                    slot.pos.h = Some( 1 );
                    slot.pos.branch_type = Some(BranchType::Dot);

                    // eprintln!("DOT bw_east {} tw_east {} bw_west {} tw_west {} h {}", slot.pos.basewidth_going_east.unwrap(), slot.pos.tipwidth_going_east.unwrap(),slot.pos.basewidth_going_west.unwrap(), slot.pos.tipwidth_going_west.unwrap(), slot.pos.h.unwrap());
                
                    left.pos.y = Some( 2 );
                    right.pos.y = Some( 2 );
                    left.pos.x = Some( 2 );
                    right.pos.x = Some( 2 );
                    return;
                    
                }
                _ => {

                    /////////////////
                    // normal binary nodes
                    let left = &mut a.leftleaf;
                    let right = &mut a.rightleaf;
                    
                    if left.pos.h == None {  set_pos(left); }
                    if right.pos.h == None {  set_pos(right);  }
                    
                    // special cases: little cones instead of trapezoids

                    // if both are cones:
                    // TODO: make this not symmetrical? (detect if the extra tip widths are on the same side, otherwise it doesn't matter)
                    if left.pos.tipwidth_going_east.unwrap() == 0 && right.pos.tipwidth_going_west.unwrap() == 0
                            {

                        // if one is a single number:
                        if left.pos.h.unwrap() == 1 || right.pos.h.unwrap() == 1 {
                            // both the cones are symmetrical

                            //AAAAAAAAAAAAAAAA
                            let leftbwidth = left.pos.basewidth_going_west.unwrap();
                            let rightbwidth = right.pos.basewidth_going_east.unwrap();
                
                            let lefttipwidth = left.pos.tipwidth_going_west.unwrap();
                            let righttipwidth = right.pos.tipwidth_going_east.unwrap();

                            slot.pos.basewidth_going_east = Some( leftbwidth + rightbwidth +2 ); // the 2 is the extra padding for the special case
                            slot.pos.basewidth_going_west = Some( leftbwidth + rightbwidth +2 ); // the 2 is the extra padding for the special case
                            slot.pos.tipwidth_going_east = Some( lefttipwidth + righttipwidth );
                            slot.pos.tipwidth_going_west = Some( lefttipwidth + righttipwidth );
                            slot.pos.h = Some( cmp::max(left.pos.h.unwrap(), right.pos.h.unwrap()) + 2 );
                            slot.pos.branch_type = Some(BranchType::Cone);

                            // eprintln!("cone1 bw_east {} tw_east {} bw_west {} tw_west {} h {}", slot.pos.basewidth_going_east.unwrap(), slot.pos.tipwidth_going_east.unwrap(),slot.pos.basewidth_going_west.unwrap(), slot.pos.tipwidth_going_west.unwrap(), slot.pos.h.unwrap ());
                        
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
                            // I've never seen the bug appear while the special case is active, but who knows!

                            let leftbwidth = left.pos.basewidth_going_west.unwrap();
                            let rightbwidth = right.pos.basewidth_going_east.unwrap();

                            let lefttipwidth = left.pos.tipwidth_going_west.unwrap();
                            let righttipwidth = right.pos.tipwidth_going_east.unwrap();
                
                            slot.pos.basewidth_going_east = Some( leftbwidth + rightbwidth + 1 );
                            slot.pos.basewidth_going_west = Some( leftbwidth + rightbwidth + 1 );
                            slot.pos.tipwidth_going_east = Some( lefttipwidth + righttipwidth );
                            slot.pos.tipwidth_going_west = Some( lefttipwidth + righttipwidth );
                            slot.pos.h = Some( cmp::max(left.pos.h.unwrap(), right.pos.h.unwrap()) + 3 );
                            slot.pos.branch_type = Some(BranchType::Cone);

                            // eprintln!("cone2 bw_east {} tw_east {} bw_west {} tw_west {} h {}", slot.pos.basewidth_going_east.unwrap(), slot.pos.tipwidth_going_east.unwrap(),slot.pos.basewidth_going_west.unwrap(), slot.pos.tipwidth_going_west.unwrap(), slot.pos.h.unwrap ());


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
                    if slot.pos.tipwidth_going_west < Some(0) { slot.pos.tipwidth_going_west = Some(0)};
                    
                    slot.pos.branch_type = Some(BranchType::Trapezoid);
                    // eprintln!("trapezoid bw_east {} tw_east {} bw_west {} tw_west {} h {}", slot.pos.basewidth_going_east.unwrap(), slot.pos.tipwidth_going_east.unwrap(),slot.pos.basewidth_going_west.unwrap(), slot.pos.tipwidth_going_west.unwrap(), slot.pos.h.unwrap ());

                    left.pos.y = Some( -4 );
                    right.pos.y = Some( -4 );
                    left.pos.x = Some( -d_between_centers/2 );
                    right.pos.x = Some( d_between_centers/2 );
                    
                }
                ///////////////////


            }
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
                        // eprintln!("error here 1 ");
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

                
                ManyNodeType::Times( times_node ) => {
                // ManyNodeType::Times( TimesNode{dice_special: Some(dsize)} ) => {

                                        
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
                        slot.pos.branch_type = Some(BranchType::Trapezoid);

                        // maybe bug with tip width like for cone2


                    } else if mslot.leaves.len() == 2 { // one time

                        let leaves = &mut mslot.leaves;

                        let tip_width_left = leaves[0].pos.tipwidth_going_west.unwrap();
                        let tip_width_right = leaves[1].pos.tipwidth_going_east.unwrap();

                        let hmin = cmp::min(leaves[0].pos.h.unwrap(), leaves[1].pos.h.unwrap());
                        let d_between_centers = tip_width_left+1 + tip_width_right+1 + 2*(hmin-1) + MIN_SPACING ;
                        // let d_between_centers = tip_width_left+1 + tip_width_right+1 + 2*(hmin-1) + MIN_SPACING +1;
                        // the +1 was added for aesthetic reasons with little understanding of what's going on!
                        // eprintln!("d_b_centers {}", d_between_centers);
            
                        slot.pos.basewidth_going_west = Some( leaves[0].pos.basewidth_going_west.unwrap() + d_between_centers );
                        slot.pos.basewidth_going_east = Some( leaves[1].pos.basewidth_going_east.unwrap() );

                        slot.pos.h = Some( cmp::max(leaves[0].pos.h.unwrap(), leaves[1].pos.h.unwrap()) + 2);

                        slot.pos.tipwidth_going_east = Some( slot.pos.basewidth_going_east.unwrap() - slot.pos.h.unwrap() +1 );
                        if slot.pos.tipwidth_going_east < Some(0) { slot.pos.tipwidth_going_east = Some(0)};
                        slot.pos.tipwidth_going_west = Some( slot.pos.basewidth_going_west.unwrap() - slot.pos.h.unwrap() +1 +2); 

                        if slot.pos.tipwidth_going_west < Some(0) { slot.pos.tipwidth_going_west = Some(0)};

                        slot.pos.branch_type = Some(BranchType::Trapezoid);
            
                        leaves[0].pos.y = Some( -2 );
                        leaves[1].pos.y = Some( -2 );
                        leaves[0].pos.x = Some( -d_between_centers );
                        leaves[1].pos.x = Some( 0 );
                        
                    // if two times, attempt to use a small cone. (2 times can still fall under "n times" below).
                    } else if mslot.leaves.len() == 3 && (mslot.leaves[1].pos.h.unwrap() == 1 || mslot.leaves[2].pos.h.unwrap() == 1 ){ // two times, with cones

                        let leaves = &mut mslot.leaves;

                        // both the cones are symmetrical
                        let leftbwidth = leaves[1].pos.basewidth_going_east.unwrap();
                        let rightbwidth = leaves[2].pos.basewidth_going_east.unwrap();
            
                        slot.pos.basewidth_going_east = Some( leftbwidth + rightbwidth +2  );
                        slot.pos.basewidth_going_west = Some( leftbwidth + rightbwidth +7  +2  );
                        slot.pos.tipwidth_going_east = Some(0);
                        slot.pos.tipwidth_going_west = Some(5); // for the "x2"
                        slot.pos.h = Some( cmp::max(leaves[1].pos.h.unwrap(), leaves[2].pos.h.unwrap()) + 2 );
                        slot.pos.branch_type = Some(BranchType::Times2Cone1);
                        
                        leaves[1].pos.y = Some( -2 );
                        leaves[2].pos.y = Some( -2 );
                        leaves[1].pos.x = Some( -2 );
                        leaves[2].pos.x = Some( 2 );

                        let dsize;
                        match times_node.dice_special {
                            Some(dsz) => dsize = dsz,
                            None => dsize = 0,
                        }
                        let dsizestep = number_w_left(dsize);
                        // leaves[0].pos.x = Some( -exprwtot/2 - d2 - MIN_SPACING  -2*dsizestep);
                        leaves[0].pos.x = Some( -6 - 2*dsizestep);
                        leaves[0].pos.y = Some( -2 );
                        return;
    
                    
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
                        for j in 1..(n_expr_leaves +1 ) {
                            
                            leaves[j].pos.x = Some( xcount);
                            leaves[j].pos.y = Some( -4 );
                            xcount += d1 + MIN_SPACING;
                        }
                        // times leaf pos
                        let tip_width_left_2 = leaves[0].pos.tipwidth_going_east.unwrap();
                        let tip_width_right_2 = leaves[1].pos.tipwidth_going_west.unwrap();
                        let hmin = cmp::min(leaves[0].pos.h.unwrap(), leaves[1].pos.h.unwrap());
                        let d2 = tip_width_left_2 +1 + tip_width_right_2 +1 + 2*(hmin-1) + MIN_SPACING;
                        

                        let dsize;
                        match times_node.dice_special {
                            Some(dsz) => dsize = dsz,
                            None => dsize = 0,
                        }
                        // cringe pos

                        let dsizestep = number_w_left(dsize);
                        leaves[0].pos.x = Some( -exprwtot/2 - d2 - MIN_SPACING  -2*dsizestep);
                        // TODO consider tweaking something, here or elsewhere, so that "2x2x2x2" is aligned again

                        leaves[0].pos.y = Some( -4 );
                        
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

// overwrite x and y as absolute position and returns max_x min_x min_y
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
        slot.pos.y = Some( newy );

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
    let oldy = slot.pos.y.unwrap();
    slot.pos.y = Some( oldy +    ytrasl );

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



// from a filled tree, draw the 2D char array for the ascii representation.
// the 2D vec likely needs to be printed to std output OR to be converted to a single string afterwards.  
fn draw_2d_vec(startslot: &mut Box<NodeSlot>) -> Vec<Vec<char>> {

    set_pos(startslot);
    startslot.pos.x = Some(0);
    startslot.pos.y = Some(0);

    let max = set_pos_abs(startslot);
    // eprintln!("max x {}", max.max_x);
    // eprintln!("min x {}", max.min_x);
    // eprintln!("max y {}", max.min_y);

    let full_w = (max.max_x - max.min_x) +1 ;

    const XLEFTPAD: usize = 4;
    const XRIGHTPAD: usize = 4;
    const YUPPAD: usize = 1;
    const YDOWNPAD: usize = 1;
    
    let vxsize = full_w as usize + XLEFTPAD + XRIGHTPAD;
    let vysize = max.min_y.abs() as usize + YUPPAD + YDOWNPAD + 1; 
    let mut v = vec![vec![' '; vxsize +1 ]; vysize];
    // eprintln!("vxsize {}", vxsize );
    // eprintln!("vysize {}", vysize );

    trasl(startslot, -max.min_x  + XLEFTPAD as i32 , -max.min_y + YUPPAD as i32 );

    write(&startslot, &mut v);
    return v;


    // wrapped function. it's inside so it's easier to make it recursive.
    fn write(slot: &Box<NodeSlot>, v: &mut Vec<Vec<char>>, ) {  
        let newx = slot.pos.x.unwrap();
        let newy = slot.pos.y.unwrap();

        let number = slot.noderesult.unwrap().value;

        let write_number_center = |number_arg: i32, x_arg: i32, y_arg: i32, vec_arg: &mut Vec<Vec<char>> | {
            let mut n = number_arg;
            if n < 0 {
                let mut stepback = 1;
                if n < -99999 {  stepback = n.to_string().len() / 2 +1; }
                else if n < -999 { stepback = 4;}
                else if n < -9 { stepback = 2;}


                vec_arg[(y_arg) as usize][(x_arg) as usize - stepback] =  '-';

                n = -n;
            }

            let offset = (n.to_string().len() / 2) as usize;
            for (i, digit) in n.to_string().chars().enumerate() {
                vec_arg[(y_arg) as usize][(x_arg + (i  as i32)) as usize - offset] =  digit;
            }
            return;
        };

        write_number_center(number, newx, newy, v);

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

                match slot.pos.branch_type {
                    Some( BranchType::Dot ) => {

                    }

                    Some( BranchType::Cone ) => {
                        let leftleaf_y = a.leftleaf.pos.y.unwrap();
                        v[(leftleaf_y) as usize][(newx) as usize] =  lchar;
                        let templen = (newy - leftleaf_y).abs();
                        for l in 1..( templen ) {

                            v[(newy - l) as usize][(newx - l ) as usize] = '\\';
                            v[(newy - l) as usize][(newx + l ) as usize] = '/';
                        }

                    }

                    Some(BranchType::Trapezoid) => {
                        let leftleaf_y = a.leftleaf.pos.y.unwrap();
                        v[(leftleaf_y) as usize][(newx) as usize] =  lchar;

                        let leftleaf_x = a.leftleaf.pos.x.unwrap();
                        let rightleaf_x = a.rightleaf.pos.x.unwrap();

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

                // Times is the only one

                let tnodex = mnode.leaves[0].pos.x.unwrap();
                let tnodey = mnode.leaves[0].pos.y.unwrap();

                // arrow
                if mnode.leaves.len() == 1 { // zero times
                    v[( tnodey + 1 ) as usize][( tnodex ) as usize] = '|';
                    v[( tnodey + 2 ) as usize][( tnodex ) as usize] = '|';

                    // v[( tnodey ) as usize][( tnodex  + 2+1) as usize] = '.';
                    // v[( tnodey ) as usize][( tnodex  + 3+1) as usize] = '.';
                    // v[( tnodey ) as usize][( tnodex  + 4+1) as usize] = '.';
                } else {

                    let first_node_x = mnode.leaves[1].pos.x.unwrap();
                    // choose arrow or ':'
                    for j in (tnodex+2)..=(first_node_x-3) {
                        v[( tnodey ) as usize][( j) as usize] = '-';
                    }
                    v[( tnodey ) as usize][( first_node_x -2 ) as usize] = '>';
                    // v[( tnodey ) as usize][( first_node_x -2 ) as usize] = ':';
                }

                match mnode.manynode {
                    //cringe draw
                    ManyNodeType::Times( TimesNode{dice_special: Some(dsize)} ) => {
                        
                        let wstep = number_w(mnode.leaves[0].noderesult.unwrap().value);
                        let wstep_dsize = number_w_left(dsize);
                        v[( tnodey) as usize][( tnodex + wstep ) as usize] = 'd';
                        write_number_center(dsize, tnodex + wstep + wstep_dsize , tnodey, v);
                    },

                    ManyNodeType::Times( TimesNode{dice_special: None} ) => {
                        v[( tnodey) as usize][( tnodex - 1) as usize] = 'x';
                    },
                }

                
                // scaffolding

                if mnode.leaves.len() == 3 && ( mnode.leaves[1].pos.h.unwrap() == 1 || mnode.leaves[2].pos.h.unwrap() == 1)  { // two times, with cones
                        let leftleaf_y = mnode.leaves[1].pos.y.unwrap();
                        let templen = (newy - leftleaf_y).abs();
                        for l in 1..( templen ) {

                            v[(newy - l) as usize][(newx - l ) as usize] = '\\';
                            v[(newy - l) as usize][(newx + l ) as usize] = '/';

                            v[(tnodey ) as usize][(newx) as usize] = '+';
                        }

                }
                else if mnode.leaves.len() >= 3 { // 2 or more times 

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

        // recursion for Many and Binary, base case at Zero.

        if let BranchType::Dot = &slot.pos.branch_type.as_ref().unwrap() {
            // skip recursion
        } else {

            match &slot.node {
                NodeType::Zero(_) => { },
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


}

// print 2D vector to terminal
fn print_2d_vec_to_terminal(v: Vec<Vec<char>>) {
    
    let termw = get_term_width();
    // eprintln!("termw {}", termw );
    let xlen = v[0].len();
    // eprintln!("xlen {}", xlen );
    // eprintln!("xlen/termw {}", xlen/termw );

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
            // eprintln!("i {} j {}", i,j);
            print!("{}",v[i][j]);
        }
        print!("\n");
    }
   
}

// print 2d vector to a single string (with proper newlines)
#[allow(dead_code)]
fn print_2d_vec_to_string(v: Vec<Vec<char>>) -> String {
    let mut res_str = String::with_capacity(v.len() * v[0].len() + v.len() ); // don't forget the newlines


    for x in 0..v.len() {


        for y in 0..v[0].len() {

            res_str.push(v[x][y]);

        }
        res_str.push('\n');
    }


    return res_str;
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

// TYPE THEORY
#[derive(Debug)]
struct NodeSlot {
    pos: NodePositionInfo,
    noderesult: Option<DiceResult>,
    node: NodeType,
}

// divided by number of leaves
#[derive(Debug)]
enum NodeType { 
    Zero(ZeroNodeSlot),
    // Unary(UnaryNode),
    Binary(BinaryNodeSlot),
    // ...
    Many(ManyNodeSlot),
}
#[derive(Debug)]
struct ZeroNodeSlot {
    zeronode: ZeroNodeType,
}
#[derive(Debug)]
struct BinaryNodeSlot {
    leftleaf: Box<NodeSlot>,
    rightleaf: Box<NodeSlot>,
    binode: BinaryNodeType, 
}
#[derive(Debug)]
struct ManyNodeSlot {
    leaves: Vec<Box<NodeSlot>>,
    manynode: ManyNodeType,
}
#[derive(Debug)]
enum ZeroNodeType {
    Number(NumberNode),
}
#[derive(Debug)]
enum BinaryNodeType {
    Addition(AdditionNode),
    Subtraction(SubtractionNode),
    Multiplication(MultiplicationNode),
    Division(DivisionNode),
    DiceRoll(DiceRollNode),
    Hit(HitNode),
}
#[derive(Debug)]
#[allow(dead_code)]
enum ManyNodeType {
    // ManySum(ManySumNode),
    Times(TimesNode),
    // Diceroll2(DiceRollNode2),
}

#[allow(dead_code)]
#[derive(Debug)]
struct NumberNode { value: i32, }
#[derive(Debug)]
struct AdditionNode { }
#[derive(Debug)]
struct SubtractionNode { }
#[derive(Debug)]
struct MultiplicationNode { }
#[derive(Debug)]
struct DivisionNode { }
#[derive(Debug)]
struct DiceRollNode { hidden: bool }
#[derive(Debug)]
struct HitNode { }

// by convention: in TimesNode, the first leaf is the number of times, the rest are summed
#[derive(Debug)]
struct TimesNode { dice_special: Option<i32> }
#[derive(Debug)]
struct DiceRollNode2 { }
// struct ManySumNode { }


fn number_w( n:i32 ) -> i32 {
    let w: i32;
    let absval = i32::abs(n);
    if absval < 99 {
        w = 1;
    } else if absval < 9999 {
        w = 2;
    } else {
        w = (((absval.to_string().len()) as i32) / 2) +1;
    }
    return w;
}

fn number_w_left( n:i32 ) -> i32 {
    let w: i32;
    let absval = i32::abs(n);
    if absval < 9 {
        w = 1;
    } else if absval < 999 {
        w = 2;
    } else {
        w = (((absval.to_string().len()) as i32) / 2) +1;
    }
    return w;
}

fn build_number(val: i32) -> NodeSlot {
    let innerinner = ZeroNodeType::Number( NumberNode{value: val} );
    let inner = ZeroNodeSlot{ zeronode: innerinner };
    let res = DiceResult{ value: val, crit: Crit::Normal };
    
    let w = number_w(val);
    
    let mut tmppos = EMPTYPOS;
    tmppos.tipwidth_going_east = Some(w as i32 - 1);
    tmppos.tipwidth_going_west = Some(w as i32 - 1);
    tmppos.basewidth_going_east = Some(w as i32 - 1);
    tmppos.basewidth_going_west = Some(w as i32 - 1);
    tmppos.h = Some( 1 );
    tmppos.branch_type = Some(BranchType::Dot);
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
#[derive(Debug)]
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
#[derive(Debug)]
enum BranchType {
    Dot,
    Cone,
    Trapezoid,
    // TimesZero,
    // TimesOne,
    TimesN,
    Times2Cone1,
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

fn dice_from_u32(dicesize: i32, random_u32: u32) -> i32 {
    let chunk_size = u32::max_value() / (dicesize as u32);
    let result = (random_u32 / chunk_size ) + 1;
    return result as i32;
}


// convert the raw string into tokens. probably the lowest-tech function in the whole program.
fn build_tokens(expr: &str) -> Result<Vec<Token>, String> { 
    
    let mut res: Vec<Token> = Vec::with_capacity(50);
    let mut number_stack = NumberStack { number_stack: Vec::new(), };

    let mut n = 0;
    let vecchar : Vec<_> = expr.chars().collect();


    while n < vecchar.len() {  

        match vecchar[n] {
            // numbers (all digits go in one token. Needs to look ahead to know if the number is finished) 
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

            // spaces
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


fn build_tree(vec: &[Token], rng: &mut rand_xoshiro::Xoroshiro128StarStar) -> Result<Box<NodeSlot>, String> {
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
                    // println!("no splitter found, got to clear parentheses maybe"); //PARSING // DEBUG
                    return build_tree(&vec[1..vec.len()-1], rng);
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



}



#[allow(dead_code)]
pub fn parse_to_string(expr: &str, rng: &mut rand_xoshiro::Xoroshiro128StarStar) -> Result<String, String> {
   
    let mut tokens = build_tokens(&expr)?;
    let spaceless_tokens = clear_spaces(&mut tokens);
    let mut tree_root = build_tree(&spaceless_tokens, rng)?;
    fill(&mut tree_root, rng);
    let char_array = draw_2d_vec(&mut tree_root);
    let end_string = print_2d_vec_to_string(char_array);
    
    return Ok(end_string);
}

#[allow(dead_code)]
pub fn parse_to_string_from_u64_seed(expr: &str, seed: u64) -> String {
    
    fn parse_with_error(expr: &str, seed: u64) -> Result<String, String> {

        let mut rng = rand_xoshiro::Xoroshiro128StarStar::seed_from_u64(seed);
        println!("did init rng");
        let mut tokens = build_tokens(&expr)?;
        println!("did tokens");
        let spaceless_tokens = clear_spaces(&mut tokens);
        println!("cleared spaces");
        let mut tree_root = build_tree(&spaceless_tokens, &mut rng)?;
        println!("buildt tree");
        fill(&mut tree_root, &mut rng);
        println!("filled tree");
        let char_array = draw_2d_vec(&mut tree_root);
        println!("drawn vec");
        let end_string = print_2d_vec_to_string(char_array);
        println!("printed string");
        
        return Ok(end_string);
    }

    // unwrap error
    match parse_with_error(expr, seed) {
        Ok(a) => return a,
        Err(b) => return b,
    }
}

pub fn parse_to_terminal(expr: &str, rng: &mut rand_xoshiro::Xoroshiro128StarStar) -> Result<(), String> {
   
    let mut tokens = build_tokens(&expr)?;
    let spaceless_tokens = clear_spaces(&mut tokens);
    let mut tree_root = build_tree(&spaceless_tokens, rng)?;
    fill(&mut tree_root, rng);
    let char_array = draw_2d_vec(&mut tree_root);
    print_2d_vec_to_terminal(char_array);
    println!("result: {}", tree_root.noderesult.unwrap().value);

    
    return Ok(());
}