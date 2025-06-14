//! Brainfuck interpreter
//!
//! # Example
//! ```
//! use brainfck::Interpreter;
//!
//! let hello_world = b">+++++++++[<++++++++>-]<.>+++++++[<++++>-]<+.+++++++..+++.[-]
//! >++++++++[<++++>-] <.>+++++++++++[<++++++++>-]<-.--------.+++
//! .------.--------.[-]>++++++++[<++++>- ]<+.[-]++++++++++.";
//!
//! let input = core::iter::empty();
//! let mut out = Vec::new();
//! let mut interpreter = Interpreter::new(hello_world, input, &mut out);
//! interpreter.run().unwrap();
//!
//! assert_eq!(out, b"Hello world!\n");
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::{
    vec::Vec,
};

use core::fmt::Display;
use core::iter::{self, Empty};

use tiny_vec::{Cow, TinyVec};

const DEFAULT_MEM_SIZE: usize = tiny_vec::n_elements_for_stack::<u8>();
const DEFAULT_PROGRAM_SIZE: usize = tiny_vec::n_elements_for_stack::<u8>();
const DEFAULT_LOOP_STACK_SIZE: usize = tiny_vec::n_elements_for_stack::<usize>();

/// Interpreter for a brainfuck program
#[derive(Clone, Debug, Default)]
pub struct Interpreter<'prog,
    In,
    Out,
    const MS: usize = DEFAULT_MEM_SIZE,
    const PS: usize = DEFAULT_PROGRAM_SIZE,
    const LS: usize = DEFAULT_LOOP_STACK_SIZE,
>
where
    In: Iterator<Item = u8>,
    Out: Writer,
{
    memory: TinyVec<u8, MS>,
    /// Mem pointer. Index for the currentñy selected memory cell
    ptr: usize,
    /// Program counter. Index of the next instruction to execute
    pc: usize,
    program: Cow<'prog, u8, PS>,
    input: In,
    output: Out,
    /// Currently open loops
    open_loops: TinyVec<usize, LS>,
    skiping_open_loop: Option<usize>,
}

impl<'prog, In, Out> Interpreter<'prog, In, Out>
where
    In: Iterator<Item = u8>,
    Out: Writer
{
    /// Builds a new brainfuck interpreter for the given `program`.
    #[inline]
    pub fn new<Prog>(program: Prog, input: In, output: Out) -> Self
    where
        Prog: Into<Cow<'prog, u8, DEFAULT_PROGRAM_SIZE>>
    {
        Interpreter::with_custom_stack(program, input, output)
    }
}

impl<'prog, In, Out,
    const MS: usize,
    const PS: usize,
    const LS: usize
> Interpreter<'prog, In, Out, MS, PS, LS>
where
    In: Iterator<Item = u8>,
    Out: Writer
{
    /// Builds a new brainfuck interpreter for the given `program`.
    pub fn with_custom_stack<Prog>(program: Prog, input: In, output: Out) -> Self
    where
        Prog: Into<Cow<'prog, u8, PS>>
    {
        let mut memory = TinyVec::<u8, MS>::new();
        memory.resize(MS, 0);
        Self {
            pc: 0,
            memory,
            program: program.into(),
            input,
            output,
            ptr: 0,
            open_loops: TinyVec::<_, LS>::new(),
            skiping_open_loop: None,
        }
    }

    /// Turns the borrowed program slice into an [owned](Cow::Owned) variant.
    ///
    /// This returns a new [Interpreter], with a `'static` lifetime
    pub fn into_owned(self) -> Interpreter<'static, In, Out, MS, PS, LS> {
        let Self { memory, ptr, pc, program, input, output, open_loops: loop_starts, skiping_open_loop: parsing_open_loop } = self;
        Interpreter {
            memory,
            program: Cow::<'static, _, PS>::Owned(program.into_owned()),
            ptr,
            pc,
            input,
            output,
            open_loops: loop_starts,
            skiping_open_loop: parsing_open_loop
        }
    }

    /// Pushes an instruction to the program
    pub fn push_instruction(&mut self, ins: u8) {
        self.program.to_mut().push(ins);
    }

    /// Pushes the given instruction slice into the program
    pub fn push_instruction_slice(&mut self, ins: &[u8]) {
        self.program.to_mut().extend_from_slice(ins);
    }

    /// Pushes all the elements yielded from `iterator`
    /// into the program
    ///
    /// # Example
    /// ```
    /// use brainfck::Interpreter;
    ///
    /// let mut bf = Interpreter::vec_output_empty_input(&[]);
    ///
    /// let ins = b"++++[>+<-]>.";
    ///
    /// bf.push_instructions_iter(ins.iter().copied());
    /// bf.run().unwrap();
    ///
    /// assert_eq!(bf.get_output(), [4]);
    ///
    /// ```
    pub fn push_instructions_iter<I>(&mut self, iterator: I)
    where
        I: Iterator<Item = u8>
    {
        self.program.to_mut().extend(iterator);
    }

    /// Executes the program until the end is reached
    pub fn run(&mut self) -> Result<(), Error> {
        while self.pc < self.program.len() {
            self.step()?;
        }
        if self.open_loops.is_empty() {
            Ok(())
        } else {
            Err(Error::OpenLoopsRemain)
        }
    }

    /// Skips a loop that was open when the pointed memory cell was 0.
    fn skip_loop(&mut self) -> Result<(), Error> {
        /* Restore the nest level, in case the program reached eof before
         * exiting the skiped loop. The program could've been extended
         * with new instructions */
        let mut nest = self.skiping_open_loop.take().unwrap_or(1);
        while nest != 0 {
            let next_ins = self.program.get(self.pc)
                .copied()
                .ok_or_else(|| {
                    /* If we reach EOF, store the current next level and return */
                    self.skiping_open_loop = Some(nest);
                    Error::OpenLoopsRemain
                })?;
            if next_ins == b'[' { nest += 1; }
            if next_ins == b']' { nest -= 1; }
            self.pc += 1;
        }
        Ok(())
    }

    /// Executes the next instruction
    pub fn step(&mut self) -> Result<(), Error> {
        /* If a skiping operation was aborted because the program didn't have
         * more instructions, resume it. */
        if self.skiping_open_loop.is_some() {
            return self.skip_loop();
        }
        if self.pc >= self.program.len() {
            return if self.open_loops.is_empty() {
                Ok(())
            } else {
                Err(Error::OpenLoopsRemain)
            }
        }
        let ins  = self.program[self.pc];
        self.pc += 1;
        match ins {
            b'>' => {
                let cap = self.memory.capacity();
                if self.ptr == cap {
                    self.memory.extend_from_slice(&[0; 100]);
                }
                self.ptr += 1;
            },
            b'<' => self.ptr -= 1,
            b'+' => self.memory[self.ptr] += 1,
            b'-' => self.memory[self.ptr] -= 1,
            b'.' => {
                let b = self.memory[self.ptr];
                self.output.push_byte(b).map_err(|_| Error::Output)?;
            },
            b',' => {
                let b = self.input.next().unwrap_or(b'\0');
                self.memory[self.ptr] = b;
            },
            b'[' => {
                if self.memory[self.ptr] == 0 {
                    self.skip_loop()?;
                } else {
                    self.open_loops.push(self.pc - 1);
                }
            },
            b']' => {
                let start = self.open_loops.pop().ok_or(Error::MissingOpenLoop)?;
                if self.memory[self.ptr] != 0 {
                    self.pc = start;
                }
            },
            c if c.is_ascii_whitespace() => {},
            _ => return Err(Error::UnexpectedByte(ins)),
        }
        Ok(())
    }

    /// Gets the current state of the program's memory
    #[inline]
    pub const fn get_memory(&self) -> &[u8] { self.memory.as_slice() }

    /// Returns true if this `Interpreter` hasn't allocated any
    /// memory on the heap
    ///
    /// The interpreter uses a [TinyVec] to store the program's memory
    /// and stack of open loops.
    /// The `TinyVec` first stores elements on the stack, until it reaches
    /// a certain length, in which it moves to the heap.
    ///
    /// If both vectors (memory and open_loops) are stack-based, and the
    /// program buffer is still borrowed, this Interpreter hasn't allocated
    /// any memory __directly__.
    pub const fn lives_on_stack(&self) -> bool {
        self.memory.lives_on_stack()
        && self.open_loops.lives_on_stack()
        && self.program.lives_on_stack()
    }
}

#[cfg(feature = "alloc")]
impl<'prog, In> Interpreter<'prog, In, Vec<u8>>
where
    In: Iterator<Item = u8>,
{
    /// Creates a new `Interpreter` with a `Vec` output
    ///
    /// This is just a shortcut for `Interpreter::new(program, input, Vec::new())`
    pub fn vec_output(program: impl Into<Cow<'prog, u8, DEFAULT_PROGRAM_SIZE>>, input: In) -> Self {
        Self::new(program, input, Vec::new())
    }

    /// Gets the output for this [`Interpreter<_, Vec<u8>>`]
    pub fn get_output(&self) -> &[u8] { &self.output }
}

impl<'prog, Out> Interpreter<'prog, Empty<u8>, Out>
where
    Out: Writer
{
    /// Creates a new `Interpreter` with an empty input
    ///
    /// This is just a shortcut for `Interpreter::new(program, iter::empty(), output)`
    pub fn empty_input(program: impl Into<Cow<'prog, u8, DEFAULT_PROGRAM_SIZE>>, output: Out) -> Self {
        Self::with_custom_stack(program, iter::empty(), output)
    }
}

#[cfg(feature = "alloc")]
impl<'prog> Interpreter<'prog, Empty<u8>, Vec<u8>> {

    /// Creates a new `Interpreter` with an empty input and a `Vec` output
    /// This is just a shortcut for `Interpreter::new(program, iter::empty(), Vec::new())`
    pub fn vec_output_empty_input(program: impl Into<Cow<'prog, u8, DEFAULT_PROGRAM_SIZE>>) -> Self {
        Self::new(program, iter::empty(), Vec::new())
    }
}

/// Signals an error on the [interpreter](Interpreter)
#[derive(Debug)]
pub enum Error {
    /// Output error. Emited from the given [writer](Writer)
    Output,
    /// Encountered a closing loop ']' that didn't match with an open '['
    MissingOpenLoop,
    /// Couldn't interpret the given byte
    UnexpectedByte(u8),
    /// EOF Reached while loops still open
    OpenLoopsRemain,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::UnexpectedByte(b) => write!(f, "Got unexpected instruction '{}' ({b})", *b as char),
            Error::Output => write!(f, "Error writing to output"),
            Error::MissingOpenLoop => write!(f, "Missing open '[' for ']'"),
            Error::OpenLoopsRemain => write!(f, "EOF reached with loops still open"),
        }
    }
}

/// Wrapper trait of [`std::io::Read`] for `no_std` enviroments.
///
/// [`std::io::Read`]: <https://doc.rust-lang.org/std/io/trait.Write.html>
pub trait Writer {
    type Error;
    fn push_byte(&mut self, byte: u8) -> Result<(), Self::Error>;
}

/// A dummy [Writer] that ignores the input and always returns [Ok]
pub struct NoOutput;

impl Writer for NoOutput {
    type Error = core::convert::Infallible;

    fn push_byte(&mut self, _byte: u8) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(feature = "std")]
impl <T: std::io::Write> Writer for T {
    type Error = std::io::Error;

    fn push_byte(&mut self, byte: u8) -> Result<(), Self::Error> {
        self.write_all(&[byte])
    }
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl<const N: usize> Writer for TinyVec<u8, N> {
    type Error = core::convert::Infallible;

    fn push_byte(&mut self, byte: u8) -> Result<(), Self::Error> {
        self.push(byte);
        Ok(())
    }
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl<const N: usize> Writer for &mut TinyVec<u8, N> {
    type Error = core::convert::Infallible;

    fn push_byte(&mut self, byte: u8) -> Result<(), Self::Error> {
        self.push(byte);
        Ok(())
    }
}

#[cfg(all(not(feature = "std"), feature = "alloc"))]
impl Writer for Vec<u8> {
    type Error = core::convert::Infallible;

    fn push_byte(&mut self, byte: u8) -> Result<(), Self::Error> {
        self.push(byte);
        Ok(())
    }
}

#[cfg(all(test, feature = "std"))]
mod test;
#[cfg(all(test, not(feature = "std")))]
mod test_no_std;
