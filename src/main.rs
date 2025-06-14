use std::{env, fs, process};
use std::io::{stdin, stdout, Read};
use brainfck::Interpreter;

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("USAGE: brainfck <program>");
        process::exit(1);
    });
    let prog = fs::read(&path).unwrap_or_else(|err| {
        eprintln!("Error reading '{path}': {err}");
        process::exit(1);
    });

    let input = stdin().lock().bytes().map_while(Result::ok);
    let out = stdout().lock();
    let mut interpreter = Interpreter::new(&prog, input, out);
    interpreter.run().unwrap_or_else(|err| {
        eprintln!("Got error: {err}");
        process::exit(1);
    });


    #[cfg(debug_assertions)]
    {
        let was_stack = interpreter.lives_on_stack();
        drop(interpreter); // Drop to free the stdout lock
        if was_stack {
            use std::io::Write;

            stdout().flush().unwrap();
            eprintln!("\n\ndone: There were no allocations for the program's memory and stack :)")
        }
    }
}
