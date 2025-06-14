use super::*;

#[test]
fn hello_world() {
    let hello_world = b">+++++++++[<++++++++>-]<.>+++++++[<++++>-]<+.+++++++..+++.[-]
        >++++++++[<++++>-] <.>+++++++++++[<++++++++>-]<-.--------.+++
        .------.--------.[-]>++++++++[<++++>- ]<+.[-]++++++++++.";

    let input = [].iter().copied();
    let mut out = Vec::new();
    let mut interpreter = Interpreter::new(hello_world, input, &mut out);
    interpreter.run().unwrap();

    assert_eq!(out, b"Hello world!\n");
}

#[test]
fn hello_world_split() {
    let prog = b">+++++++++[<++++++++>-]<.>+++++++[<++++>-]<+.+++++++..+++.[-]
        >++++++++[<++++>-] <.>+++++++++++[<++++++";

    let half = b"++>-]<-.--------.+++
        .------.--------.[-]>++++++++[<++++>- ]<+.[-]++++++++++.";

    let mut out = Vec::new();
    let mut interpre = Interpreter::new(prog, [].iter().copied(),  &mut out);

    interpre.run().expect_err("OPEN LOOPS");
    interpre.push_instructions(half);
    interpre.run().unwrap();

    assert_eq!(out, b"Hello world!\n");
}
