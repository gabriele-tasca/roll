use rustyline::error::ReadlineError;
use rustyline::Editor;

use rand_xoshiro::rand_core::SeedableRng;

use std::error::Error;

mod parse_functions;


const PROGRAM_NAME: &str = "roll";
const HIST_FILE_NAME: &str = "history";
const HIST_FILE_EXT: &str = "txt";


fn main() -> Result<(), Box<dyn Error>>  {

    let mut rng = rand_xoshiro::Xoroshiro128StarStar::seed_from_u64(729);

    let mut rl = Editor::<()>::new();

    // load history      
    let mut load_history_from_right_place = || -> Result<(), Box<dyn Error>>  { 
        let mut path = dirs::data_local_dir().ok_or("error")?;            
        path.push(PROGRAM_NAME);
        path.push(HIST_FILE_NAME);
        path.set_extension(HIST_FILE_EXT);
        rl.load_history(&path)?;
        Ok(())  
    };
    if let Err(_) = load_history_from_right_place() {
        if !rl.load_history("history.txt").is_err() {
            println!("No history found");
        }
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {

                if line == "exit" || line == "Exit" || line == "quit" || line == "Quit" || line == "q" {
                    break
                }

                rl.add_history_entry(line.as_str());

                match &mut parse_functions::parse_to_terminal(&line, &mut rng) {
                    Err(a) => println!("{}",a),
                    Ok(_b) => { },
                }
                // println!("{}", parse_functions::parse_to_string_from_u64_seed(&line, 729) );


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


    // save history in right place     
    let save_history_in_right_place = || -> Result<(), Box<dyn Error>>  { 
        let mut path = dirs::data_local_dir().ok_or("error")?;            
        path.push(PROGRAM_NAME);
        std::fs::create_dir_all(&path)?;
        path.push(HIST_FILE_NAME);
        path.set_extension(HIST_FILE_EXT);
        rl.save_history(&path)?;
        Ok(())  
    };
    if let Err(_) = save_history_in_right_place() {
        println!("cannot find the right directory, printing history in current working dir");
        rl.save_history("history.txt").unwrap();
    }

    Ok(())



    

    // FREQUENCY TEST
    
    // let big_number = 100000;
    // let expression = "1d20";  // MAKE THE DICE SIZE MATCH THE VEC! LEN
    // let mut counter = vec![0;20];
    // for _i in 0..big_number {

    //         let expression = expression.trim();
    //         let mut restokens = build_tokens(&expression);
    //         let mut res = 0;
    //         match &mut restokens {
    //             Err(a) => println!("{}",a),
    //             Ok(b) => {
    //                 let tokens = clear_spaces(b);
    
    //                 match parse_to_terminal_1(tokens, &mut rng) {
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