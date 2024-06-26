use core::fmt;
use std::collections::HashMap;

use anyhow::{Error, Result};

use crate::codegen::{ByteCodeFunction, ByteCodeOp, ByteCodeValue};

#[derive(Debug)]
pub struct Runtime {
    operations: Vec<ByteCodeOp>,
    pc: usize,
    call_stack: Vec<usize>,
    value_stack: Vec<ByteCodeValue>,
    ftxc_stack: Vec<HashMap<usize, ByteCodeValue>>,
    label_offsets: HashMap<String, usize>,
}

impl std::fmt::Display for Runtime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "--------------")?;
        writeln!(f, "Pc: {}", self.pc)?;
        writeln!(f, "Op[PC]: {:?}", self.operations[self.pc])?;
        writeln!(f, "CallStack: {:?}", self.call_stack)?;
        writeln!(f, "ValueStack: {:?}", self.value_stack)?;
        writeln!(f, "FctxSStack: {:?}", self.ftxc_stack)?;
        writeln!(f, "--------------")
    }
}

impl Runtime {
    pub fn new(function_list: Vec<ByteCodeFunction>) -> Self {
        let mut label_offsets = HashMap::new();
        let mut pc = 0;
        let operations = function_list
            .iter()
            .flat_map(|function| &function.ops)
            .enumerate()
            .map(|(i, op)| {
                if let ByteCodeOp::Label(ref label) = op.bytecode_op {
                    if label == "main" {
                        pc = i;
                    }
                    label_offsets.insert(label.clone(), i);
                }
                op.bytecode_op.clone()
            })
            .collect();
        Runtime {
            operations,
            pc,
            call_stack: Vec::new(),
            value_stack: Vec::new(),
            ftxc_stack: vec![HashMap::new()],
            label_offsets,
        }
    }

    fn push_next(&mut self, val: ByteCodeValue) {
        self.pc += 1;
        self.value_stack.push(val);
    }

    pub fn execute_program(&mut self) -> Result<usize, Error> {
        while self.operations[self.pc] != ByteCodeOp::End {
            // println!("{}", self);
            // println!("Call {:?}", self.call_stack);
            // println!("Val {:?}", self.value_stack);
            // println!("Fctx {:?}", self.ftxc_stack);

            match &self.operations[self.pc] {
                ByteCodeOp::Return => {
                    self.pc = self.call_stack.pop().unwrap();
                    self.ftxc_stack.pop();
                    let ret = self.value_stack.pop().unwrap();
                    while Some(ByteCodeValue::Return) != self.value_stack.pop() {}
                    self.value_stack.push(ret);
                }
                ByteCodeOp::LocalGet(index) => {
                    let Some(val) = self.ftxc_stack.last().unwrap().get(index) else {
                        panic!("RT LocalGet variable not found");
                    };
                    self.push_next(val.clone());
                }
                ByteCodeOp::LocalSet(index) => {
                    let Some(value) = self.value_stack.pop() else {
                        panic!("RT Local Set empty satck");
                    };
                    self.ftxc_stack.last_mut().unwrap().insert(*index, value);
                    self.pc += 1;
                }
                ByteCodeOp::Const(val) => {
                    self.push_next(val.clone());
                }
                ByteCodeOp::Add => {
                    let Some(ByteCodeValue::Number(a)) = self.value_stack.pop() else {
                        panic!("RT Add received non number");
                    };
                    let Some(ByteCodeValue::Number(b)) = self.value_stack.pop() else {
                        panic!("RT Add received non number");
                    };
                    self.push_next(ByteCodeValue::Number(a + b))
                }
                ByteCodeOp::Sub => {
                    let Some(ByteCodeValue::Number(a)) = self.value_stack.pop() else {
                        panic!("RT Sub received non number");
                    };
                    let Some(ByteCodeValue::Number(b)) = self.value_stack.pop() else {
                        panic!("RT Sub received non number");
                    };
                    self.push_next(ByteCodeValue::Number(b - a))
                }
                ByteCodeOp::Div => {
                    let Some(ByteCodeValue::Number(a)) = self.value_stack.pop() else {
                        panic!("RT Div received non number");
                    };
                    let Some(ByteCodeValue::Number(b)) = self.value_stack.pop() else {
                        panic!("RT Div received non number");
                    };
                    self.push_next(ByteCodeValue::Number(a / b))
                }
                ByteCodeOp::Mul => {
                    let Some(ByteCodeValue::Number(a)) = self.value_stack.pop() else {
                        panic!("RT Mul received non number");
                    };
                    let Some(ByteCodeValue::Number(b)) = self.value_stack.pop() else {
                        panic!("RT Mul received non number");
                    };
                    self.push_next(ByteCodeValue::Number(a * b));
                }
                ByteCodeOp::ListAt => panic!("Listen machma spaeter"),
                ByteCodeOp::LowerT => {
                    let Some(ByteCodeValue::Number(a)) = self.value_stack.pop() else {
                        panic!("RT LowerT received non number");
                    };
                    let Some(ByteCodeValue::Number(b)) = self.value_stack.pop() else {
                        panic!("RT LowerT received non number");
                    };
                    self.push_next(ByteCodeValue::Boolean(b < a))
                }
                ByteCodeOp::GreaterT => {
                    let Some(ByteCodeValue::Number(a)) = self.value_stack.pop() else {
                        panic!("RT GreaterT received non number");
                    };
                    let Some(ByteCodeValue::Number(b)) = self.value_stack.pop() else {
                        panic!("RT GreaterT received non number");
                    };
                    self.push_next(ByteCodeValue::Boolean(b > a))
                }
                ByteCodeOp::Equal => {
                    let Some(a) = self.value_stack.pop() else {
                        panic!("RT Equal received non number");
                    };
                    let Some(b) = self.value_stack.pop() else {
                        panic!("RT Equal received non number");
                    };
                    self.push_next(ByteCodeValue::Boolean(a == b))
                }
                ByteCodeOp::NotEq => {
                    let Some(a) = self.value_stack.pop() else {
                        panic!("RT NotEq received non number");
                    };
                    let Some(b) = self.value_stack.pop() else {
                        panic!("RT NotEq received non number");
                    };
                    self.push_next(ByteCodeValue::Boolean(a != b))
                }
                ByteCodeOp::Call(funcname, argc) => {
                    let mut new_fctx = HashMap::new();
                    for i in (0..*argc).rev() {
                        let Some(val) = self.value_stack.pop() else {
                            panic!("Nothing left to pop for call");
                        };
                        new_fctx.insert(i, val);
                    }
                    self.value_stack.push(ByteCodeValue::Return);
                    self.ftxc_stack.push(new_fctx);
                    self.call_stack.push(self.pc + 1);
                    self.pc = *self.label_offsets.get(funcname).unwrap();
                }
                ByteCodeOp::Print => {
                    let Some(value) = self.value_stack.pop() else {
                        panic!("RT Add received non number");
                    };
                    println!("{}", value);
                    self.pc += 1;
                }
                ByteCodeOp::JumpTrue(label) => {
                    if let Some(ByteCodeValue::Boolean(true)) = self.value_stack.pop() {
                        self.pc = *self.label_offsets.get(label).unwrap()
                    } else {
                        self.pc += 1;
                    }
                }
                ByteCodeOp::JumpFalse(label) => {
                    if let Some(ByteCodeValue::Boolean(false)) = self.value_stack.pop() {
                        self.pc = *self.label_offsets.get(label).unwrap()
                    } else {
                        self.pc += 1;
                    }
                }
                ByteCodeOp::Label(_) => {
                    self.pc += 1;
                }
                ByteCodeOp::End => break,
                ByteCodeOp::Jump(label) => self.pc = *self.label_offsets.get(label).unwrap(),
                ByteCodeOp::Pop => {
                    self.value_stack.pop();
                    self.pc += 1;
                }
            }
        }
        Ok(0)
    }
}
