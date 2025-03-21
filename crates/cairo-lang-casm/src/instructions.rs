#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};
use core::fmt::Display;

use crate::hints::{Hint, PythonicHint};
use crate::operand::{CellRef, DerefOrImmediate, ResOperand};

#[cfg(test)]
#[path = "instructions_test.rs"]
mod test;

// An enum of Cairo instructions.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum InstructionBody {
    AddAp(AddApInstruction),
    AssertEq(AssertEqInstruction),
    QM31AssertEq(AssertEqInstruction),
    Call(CallInstruction),
    Jnz(JnzInstruction),
    Jump(JumpInstruction),
    Ret(RetInstruction),
    Blake2sCompress(Blake2sCompressInstruction),
}
impl InstructionBody {
    pub fn op_size(&self) -> usize {
        // TODO(spapini): Make this correct.
        match self {
            InstructionBody::AddAp(insn) => insn.op_size(),
            InstructionBody::AssertEq(insn) | InstructionBody::QM31AssertEq(insn) => insn.op_size(),
            InstructionBody::Call(insn) => insn.op_size(),
            InstructionBody::Jump(insn) => insn.op_size(),
            InstructionBody::Jnz(insn) => insn.op_size(),
            InstructionBody::Ret(insn) => insn.op_size(),
            InstructionBody::Blake2sCompress(insn) => insn.op_size(),
        }
    }
}
impl Display for InstructionBody {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InstructionBody::AddAp(insn) => write!(f, "{insn}",),
            InstructionBody::AssertEq(insn) => write!(f, "{insn}",),
            InstructionBody::QM31AssertEq(insn) => write!(f, "{{QM31}} {insn}",),
            InstructionBody::Call(insn) => write!(f, "{insn}",),
            InstructionBody::Jnz(insn) => write!(f, "{insn}",),
            InstructionBody::Jump(insn) => write!(f, "{insn}",),
            InstructionBody::Ret(insn) => write!(f, "{insn}",),
            InstructionBody::Blake2sCompress(insn) => write!(f, "{insn}",),
        }
    }
}

/// Represents an instruction, including the ap++ flag (inc_ap).
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Instruction {
    pub body: InstructionBody,
    pub inc_ap: bool,
    pub hints: Vec<Hint>,
}
impl Instruction {
    pub fn new(body: InstructionBody, inc_ap: bool) -> Self {
        Self { body, inc_ap, hints: vec![] }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for hint in &self.hints {
            let hint_str = hint.get_pythonic_hint();
            // Skip leading and trailing space if hint starts with `\n`.
            if hint_str.starts_with('\n') {
                writeln!(f, "%{{{hint_str}%}}")
            } else {
                writeln!(f, "%{{ {hint_str} %}}")
            }?
        }

        write!(f, "{}", self.body)?;
        if self.inc_ap {
            write!(f, ", ap++")?
        };
        Ok(())
    }
}

/// Represents a call instruction "call rel/abs target".
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CallInstruction {
    pub target: DerefOrImmediate,
    pub relative: bool,
}
impl Display for CallInstruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "call {} {}", if self.relative { "rel" } else { "abs" }, self.target,)
    }
}
impl CallInstruction {
    pub fn op_size(&self) -> usize {
        match &self.target {
            DerefOrImmediate::Deref(_) => 1,
            DerefOrImmediate::Immediate(_) => 2,
        }
    }
}

/// Represents the InstructionBody "jmp rel/abs target".
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct JumpInstruction {
    pub target: DerefOrImmediate,
    pub relative: bool,
}
impl JumpInstruction {
    pub fn op_size(&self) -> usize {
        match &self.target {
            DerefOrImmediate::Deref(_) => 1,
            DerefOrImmediate::Immediate(_) => 2,
        }
    }
}
impl Display for JumpInstruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "jmp {} {}", if self.relative { "rel" } else { "abs" }, self.target,)
    }
}

/// Represents the InstructionBody "jmp rel <jump_offset> if condition != 0".
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct JnzInstruction {
    pub jump_offset: DerefOrImmediate,
    pub condition: CellRef,
}
impl JnzInstruction {
    pub fn op_size(&self) -> usize {
        match &self.jump_offset {
            DerefOrImmediate::Deref(_) => 1,
            DerefOrImmediate::Immediate(_) => 2,
        }
    }
}
impl Display for JnzInstruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "jmp rel {} if {} != 0", self.jump_offset, self.condition)
    }
}

/// Returns the size of instruction based on whether the res operand includes an immediate or not.
pub fn op_size_based_on_res_operands(operand: &ResOperand) -> usize {
    match operand {
        ResOperand::Deref(_) => 1,
        ResOperand::DoubleDeref(_, _) => 1,
        ResOperand::Immediate(_) => 2,
        ResOperand::BinOp(op) => match op.b {
            DerefOrImmediate::Immediate(_) => 2,
            DerefOrImmediate::Deref(_) => 1,
        },
    }
}

/// Represents the InstructionBody "a = b" for two operands a, b.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AssertEqInstruction {
    pub a: CellRef,
    pub b: ResOperand,
}
impl AssertEqInstruction {
    pub fn op_size(&self) -> usize {
        op_size_based_on_res_operands(&self.b)
    }
}
impl Display for AssertEqInstruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} = {}", self.a, self.b)
    }
}

/// Represents a return instruction, "ret".
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RetInstruction {}
impl Display for RetInstruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ret")
    }
}

impl RetInstruction {
    pub fn op_size(&self) -> usize {
        1
    }
}

/// Represents the InstructionBody "ap += op" for a given operand op.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AddApInstruction {
    pub operand: ResOperand,
}
impl AddApInstruction {
    pub fn op_size(&self) -> usize {
        op_size_based_on_res_operands(&self.operand)
    }
}
impl Display for AddApInstruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ap += {}", self.operand)
    }
}

/// Represents a blake2s instruction, "blake2s".
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Blake2sCompressInstruction {
    pub state: CellRef,
    pub byte_count: CellRef,
    pub message: CellRef,
    pub finalize: bool,
}
impl Blake2sCompressInstruction {
    pub fn op_size(&self) -> usize {
        1
    }
}
impl Display for Blake2sCompressInstruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "blake2s[state={}, message={}, byte_count={}, finalize={}] => [ap + 0]",
            self.state, self.message, self.byte_count, self.finalize
        )
    }
}
