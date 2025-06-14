use super::*;

#[test]
fn hello_world() {
    let hello_world = b">+++++++++[<++++++++>-]<.>+++++++[<++++>-]<+.+++++++..+++.[-]
        >++++++++[<++++>-] <.>+++++++++++[<++++++++>-]<-.--------.+++
        .------.--------.[-]>++++++++[<++++>- ]<+.[-]++++++++++.";

    let mut interpreter = Interpreter::vec_output_empty_input(hello_world);
    interpreter.run().unwrap();

    assert_eq!(interpreter.get_output(), b"Hello world!\n");
}

#[test]
fn hello_world_split() {
    let prog = b">+++++++++[<++++++++>-]<.>+++++++[<++++>-]<+.+++++++..+++.[-]
        >++++++++[<++++>-] <.>+++++++++++[<++++++";

    let half = b"++>-]<-.--------.+++
        .------.--------.[-]>++++++++[<++++>- ]<+.[-]++++++++++.";

    let mut interpre = Interpreter::vec_output_empty_input(prog);

    interpre.run().expect_err("OPEN LOOPS");
    interpre.push_instruction_slice(half);
    interpre.run().unwrap();

    assert_eq!(interpre.get_output(), b"Hello world!\n");
}

#[test]
fn split_loop() {
    let prog = b">++<[+++";
    let half = b"++++]+[-+";
    let last = b"-]++++++++++.";
    let mut out = Vec::new();
    let mut interpre = Interpreter::new(prog, [].iter().copied(),  &mut out);

    /* We've split on an open loop that was being skipped
     * (because the memory at that point was 0).
     * So the skip loop will break, and we need to check that after
     * pushing the rest of the program, nothing breaks. */
    match interpre.run() {
        Err(Error::OpenLoopsRemain) => {},
        _ => panic!("Expected an \"OpenLoopsRemainError\" error"),
    }

    interpre.push_instruction_slice(half);

    /* Here we break, but the loop was being processed. This is easier, since
     * we don't need to store any "skipping" state, and just keep running */
    match interpre.run() {
        Err(Error::OpenLoopsRemain) => {},
        _ => panic!("Expected an \"OpenLoopsRemainError\" error"),
    }
    interpre.push_instruction_slice(last);
    interpre.run().unwrap();

    assert_eq!(out, b"\n");
}

#[test]
fn push_iter() {
    let ins = b"++++[>+<-]>.";

    let mut out = Vec::new();
    let mut interpre = Interpreter::new(&[], [].iter().copied(),  &mut out);

    interpre.push_instructions_iter(ins.iter().copied());
    interpre.run().unwrap();

    assert_eq!(out, [4]);
}

#[test]
fn tiny_vec_test_mem() {
    let prog = b">>>>>>>>>>>>>>>>>";

    let mut inter = Interpreter::vec_output_empty_input(prog);

    for _ in 0..MEM_TINY_SIZE {
        inter.step().unwrap();
    }
    assert!(inter.lives_on_stack());

    // This step moves the mem pointer to the index (MEM_TINY_SIZE + 1)
    // This will cause the TinyVec to resize, and move to the heap
    inter.step().unwrap();

    assert!(!inter.lives_on_stack());

    inter.run().unwrap();
}

#[test]
fn tiny_vec_test_loops() {
    let prog = b"+[[[-]]]";

    let mut inter = Interpreter::vec_output_empty_input(prog);

    inter.step().unwrap(); // the '+'
    for _ in 0..OPEN_LOOPS_TINY_SIZE {
        inter.step().unwrap();
    }
    assert!(inter.lives_on_stack());

    // This step pushes an index into open_loops that causes the
    // the TinyVec to resize, and move to the heap
    inter.step().unwrap();

    assert!(!inter.lives_on_stack());

    inter.run().unwrap();
}

#[test]
fn tiny_vec_test_prog_owned() {
    let prog = b"+++";

    let mut inter = Interpreter::vec_output_empty_input(prog);

    assert!(inter.lives_on_stack());

    // This pushes a byte to the program, causng it to turn
    // into an Owned variant, and allocating memory.
    inter.push_instruction(b'.');
    assert!(!inter.lives_on_stack());

    inter.run().unwrap();
}
