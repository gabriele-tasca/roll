//el backuppo haha xd

fn draw_2d_vec(startslot: &mut Box<NodeSlot>) -> Vec<Vec<char>> {

    set_pos(startslot);
    startslot.pos.x = Some(0);
    startslot.pos.y = Some(0);

    eprintln!("set_pos successful");

    //size array
    let mut v = vec![vec![' '; 150]; 35];
    let startx = 75;
    let starty = 35 - 1;

    let max = set_pos_abs(startslot);

    dive(&startslot, &mut v, startx, starty);


    println!("max x {}", max.max_x);
    println!("min x {}", max.min_x);
    println!("max y {}", max.max_y);

    return v;




    fn dive(slot: &Box<NodeSlot>, v: &mut Vec<Vec<char>>, lastx: i32, lasty: i32, ) {
        let this_rel_x = slot.pos.x.unwrap();
        let this_rel_y = slot.pos.y.unwrap();
    
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


}