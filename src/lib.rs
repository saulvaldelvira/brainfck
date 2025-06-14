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
//! let input = [].iter().copied();
//! let mut out = Vec::new();
//! let mut interpreter = Interpreter::new(hello_world, input, &mut out);
//! interpreter.run().unwrap();
//!
//! assert_eq!(out, b"Hello world!\n");
//! ```

use core::fmt::Display;
use std::borrow::Cow;
use std::io;

/// Interpreter for a brainfuck program
pub struct Interpreter<'prog, In, Out>
where
    In: Iterator<Item = u8>,
    Out: io::Write,
{
    memory: Vec<u8>,
    ptr: usize,
    pc: usize,
    program: Cow<'prog, [u8]>,
    input: In,
    output: Out,
    loop_starts: Vec<usize>,
}

impl<'prog, In, Out> Interpreter<'prog, In, Out>
where
    In: Iterator<Item = u8>,
    Out: io::Write,
{
    /// Builds a new brainfuck interpreter for the given `program`.
    pub fn new(program: impl Into<Cow<'prog, [u8]>>, input: In, output: Out) -> Self {
        let memory = vec![0; 1000];
        Self {
            pc: 0,
            memory,
            program: program.into(),
            input,
            output,
            ptr: 0,
            loop_starts: Vec::new(),
        }
    }

    /// Turns the borrowed program slice into an [owned](Cow::Owned) variant.
    ///
    /// This returns a new [Interpreter], with a `'static` lifetime
    pub fn into_owned(self) -> Interpreter<'static, In, Out> {
        let Self { memory, ptr, pc, program, input, output, loop_starts } = self;
        Interpreter {
            memory,
            program: Cow::<'static, _>::Owned(program.into_owned()),
            ptr,
            pc,
            input,
            output,
            loop_starts,
        }
    }

    /// Pushes more instructions to the program
    pub fn push_instructions(&mut self, ins: &[u8]) {
        self.program.to_mut().extend_from_slice(ins);
    }

    /// Executes the program until the end is reached
    pub fn run(&mut self) -> Result<(), Error> {
        while self.pc < self.program.len() {
            self.step()?;
        }
        if self.loop_starts.is_empty() {
            Ok(())
        } else {
            Err(Error::OpenLoopsRemain)
        }
    }

    /// Executes the next instruction
    pub fn step(&mut self) -> Result<(), Error> {
        if self.pc >= self.program.len() {
            return if self.loop_starts.is_empty() {
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
                    self.memory.extend_from_slice(&[0; 1000]);
                }
                self.ptr += 1;
            },
            b'<' => self.ptr -= 1,
            b'+' => self.memory[self.ptr] += 1,
            b'-' => self.memory[self.ptr] -= 1,
            b'.' => {
                let b = self.memory[self.ptr];
                self.output.write_all(&[b])?;
            },
            b',' => {
                let b = self.input.next().unwrap_or(b'\0');
                self.memory[self.ptr] = b;
            },
            b'[' => {
                if self.memory[self.ptr] == 0 {
                    let mut nest = 1;
                    while nest != 0 {
                        self.pc += 1;
                        let next_ins = self.program[self.pc];
                        if next_ins == b'[' { nest += 1; }
                        if next_ins == b']' { nest -= 1; }
                    }
                } else {
                    self.loop_starts.push(self.pc - 1);
                }
            },
            b']' => {
                let start = self.loop_starts.pop().ok_or(Error::MissingOpenLoop)?;
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
    pub fn get_memory(&self) -> &[u8] { &self.memory }
}

#[derive(Debug)]
pub enum Error {
    Output(io::Error),
    MissingOpenLoop,
    UnexpectedByte(u8),
    OpenLoopsRemain,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnexpectedByte(b) => write!(f, "Got unexpected instruction '{}' ({b})", *b as char),
            Error::Output(error) => write!(f, "{error}"),
            Error::MissingOpenLoop => write!(f, "Missing open '[' for ']'"),
            Error::OpenLoopsRemain => write!(f, "EOF reached with loops still open"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Output(value)
    }
}

#[cfg(test)]
mod test;
