use super::*;

#[test]
fn no_std_test() {
    let ins = b"++++[>+<-]>.";

    let mut out = TinyVec::<_, 10>::new();
    let mut interpre = Interpreter::new(&[], [].iter().copied(),  &mut out);

    interpre.push_instructions_iter(ins.iter().copied());
    interpre.run().unwrap();

    assert_eq!(out, [4]);
}

#[test]
#[should_panic(expected = "Alloc is not enabled. Can't switch the buffer to the heap")]
fn default_size_panics() {
    let ins = b"++++[>+<-]>.>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>";

    let mut out = TinyVec::<_, 10>::new();
    let mut interpre = Interpreter::new(&[], [].iter().copied(),  &mut out);

    interpre.push_instructions_iter(ins.iter().copied());
    interpre.run().unwrap();

    assert_eq!(out, [4]);
}

#[test]
fn custom_size_no_panic() {
    let ins = b"++++[>+<-]>.>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>";

    let mut out = TinyVec::<_, 10>::new();
    let mut interpre = Interpreter::<'_, _, _, 100, 200, 100>::with_custom_stack(&[], [].iter().copied(),  &mut out);

    interpre.push_instructions_iter(ins.iter().copied());
    interpre.run().unwrap();

    assert_eq!(out, [4]);
}

#[test]
fn tiny_vec_test_loops() {
    let prog = b"+[[[[[-]]]]]";

    let mut inter = Interpreter::<'_, _, _>::with_custom_stack(prog, [].iter().copied(),  NoOutput);

    inter.step().unwrap(); // the '+'
    for _ in 0..OPEN_LOOPS_TINY_SIZE {
        inter.step().unwrap();
    }
}

#[test]
#[should_panic]
fn tiny_vec_test_loops_panic() {
    let prog = b"+[[[[[[[-]]]]]";

    let mut inter = Interpreter::<'_, _, _>::with_custom_stack(prog, [].iter().copied(),  NoOutput);

    inter.step().unwrap(); // the '+'
    for _ in 0..OPEN_LOOPS_TINY_SIZE {
        inter.step().unwrap();
    }

    inter.step().unwrap();
}
