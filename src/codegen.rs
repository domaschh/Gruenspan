use crate::parser::{BinaryOp, Expr, Func, Value};
use anyhow::{bail, Result};
use std::{
    collections::HashMap,
    fmt::{self, format},
    path::Display,
    pin::Pin,
};

#[derive(Debug)]
pub struct RelativeOperation {
    pub bytecode_op: ByteCodeOp,
}

impl RelativeOperation {
    fn new(bop_type: ByteCodeOp) -> Self {
        RelativeOperation {
            bytecode_op: bop_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ByteCodeValue {
    Number(f64),
    Boolean(bool),
    String(String),
    List(Vec<ByteCodeValue>),
    Return,
}

impl fmt::Display for ByteCodeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ByteCodeValue::Number(v) => write!(f, "{}", v),
            ByteCodeValue::Boolean(v) => write!(f, "{}", v),
            ByteCodeValue::String(v) => write!(f, "{}", v),
            ByteCodeValue::List(v) => write!(f, "{:?}", v),
            ByteCodeValue::Return => write!(f, "Return"),
        }
    }
}

impl From<&Value> for ByteCodeValue {
    fn from(value: &Value) -> Self {
        match value {
            Value::Null => panic!("Wtf converstion from &Val to BopVal failed"),
            Value::Bool(b) => ByteCodeValue::Boolean(*b),
            Value::Num(n) => ByteCodeValue::Number(*n),
            Value::Str(sr) => ByteCodeValue::String(sr.clone()),
            Value::List(l) => ByteCodeValue::List(l.into_iter().map(|a| a.into()).collect()),
            Value::Func(fp) => panic!("Wtf converstion from &Val to BopVal failed"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ByteCodeOp {
    Return,
    LocalGet(usize),
    LocalSet(usize),
    Const(ByteCodeValue),
    Add,
    Sub,
    Div,
    Mul,
    ListAt,
    LowerT,
    GreaterT,
    Equal,
    NotEq,
    Call(String, usize),
    Print,
    Jump(String),
    JumpTrue(String),
    JumpFalse(String),
    Label(String),
    End,
}

#[derive(Debug)]
pub struct ByteCodeFunction {
    pub name: String,
    pub ops: Vec<RelativeOperation>,
    pub arg_ct: usize,
}

impl ByteCodeFunction {
    fn new(name: String, ops: Vec<RelativeOperation>, arg_ct: usize) -> Self {
        ByteCodeFunction { name, ops, arg_ct }
    }
}

fn generate_function_bytecode(
    expr: &Expr,
    mut store_ct: usize,
    mut label_ctr: usize,
    method_name: &str,
    mem_store: &mut HashMap<String, usize>,
    operations: &mut Vec<RelativeOperation>,
) {
    match expr {
        Expr::Error => unreachable!(),
        Expr::Value(val) => match val {
            Value::Null => {}
            Value::Bool(bool) => operations.push(RelativeOperation::new(ByteCodeOp::Const(
                ByteCodeValue::Boolean(*bool),
            ))),
            Value::Num(num) => operations.push(RelativeOperation::new(ByteCodeOp::Const(
                ByteCodeValue::Number(*num),
            ))),
            Value::Str(str) => operations.push(RelativeOperation::new(ByteCodeOp::Const(
                ByteCodeValue::String(str.clone()),
            ))),
            Value::List(list) => operations.push(RelativeOperation::new(ByteCodeOp::Const(
                ByteCodeValue::List(list.into_iter().map(|val| val.into()).collect()),
            ))),
            Value::Func(fp) => println!("When am I called {:?}", fp),
        },
        Expr::List(_) => todo!(),
        Expr::LocalVar(varname) => operations.push(RelativeOperation::new(ByteCodeOp::LocalGet(
            *mem_store.get(varname).unwrap(),
        ))),
        Expr::Let(variable, expression, other) => {
            generate_function_bytecode(
                &(**expression).0,
                store_ct,
                label_ctr,
                method_name,
                mem_store,
                operations,
            );
            mem_store.insert(variable.clone(), store_ct);
            operations.push(RelativeOperation::new(ByteCodeOp::LocalSet(store_ct)));
            store_ct += 1;
            generate_function_bytecode(
                &(**other).0,
                store_ct,
                label_ctr,
                method_name,
                mem_store,
                operations,
            );
        }
        Expr::Then(this_expr, next_expr) => {
            generate_function_bytecode(
                &(**this_expr).0,
                store_ct,
                label_ctr,
                method_name,
                mem_store,
                operations,
            );
            generate_function_bytecode(
                &(**next_expr).0,
                store_ct,
                label_ctr,
                method_name,
                mem_store,
                operations,
            );
        }
        Expr::Binary(lhs, operation, rhs) => {
            generate_function_bytecode(
                &(**lhs).0,
                store_ct,
                label_ctr,
                method_name,
                mem_store,
                operations,
            );
            generate_function_bytecode(
                &(**rhs).0,
                store_ct,
                label_ctr,
                method_name,
                mem_store,
                operations,
            );
            match operation {
                BinaryOp::Add => operations.push(RelativeOperation::new(ByteCodeOp::Add)),
                BinaryOp::Sub => operations.push(RelativeOperation::new(ByteCodeOp::Sub)),
                BinaryOp::Mul => operations.push(RelativeOperation::new(ByteCodeOp::Mul)),
                BinaryOp::Div => operations.push(RelativeOperation::new(ByteCodeOp::Div)),
                BinaryOp::Eq => operations.push(RelativeOperation::new(ByteCodeOp::Equal)),
                BinaryOp::NotEq => operations.push(RelativeOperation::new(ByteCodeOp::NotEq)),
                BinaryOp::LowerT => operations.push(RelativeOperation::new(ByteCodeOp::LowerT)),
                BinaryOp::GreaterT => operations.push(RelativeOperation::new(ByteCodeOp::GreaterT)),
                BinaryOp::ListAt => operations.push(RelativeOperation::new(ByteCodeOp::ListAt)),
            }
        }
        Expr::Call(func_name, arguments) => {
            for arg in arguments.0.iter() {
                generate_function_bytecode(
                    &arg.0,
                    store_ct,
                    label_ctr,
                    method_name,
                    mem_store,
                    operations,
                );
            }
            let Expr::LocalVar(funcname_vale) = &func_name.0 else {
                panic!("Funcname not string");
            };

            operations.push(RelativeOperation::new(ByteCodeOp::Call(
                funcname_vale.clone(),
                arguments.0.len(),
            )));
        }
        Expr::If(cond, then, els) => {
            let increased_labelctr = label_ctr + 1;
            generate_function_bytecode(
                &(**cond).0,
                store_ct,
                increased_labelctr,
                method_name,
                mem_store,
                operations,
            );
            operations.push(RelativeOperation::new(ByteCodeOp::JumpFalse(format!(
                "{}_{}_{}",
                method_name, "else", label_ctr
            ))));
            generate_function_bytecode(
                &(**then).0,
                store_ct,
                increased_labelctr,
                method_name,
                mem_store,
                operations,
            );
            operations.push(RelativeOperation::new(ByteCodeOp::Jump(format!(
                "{}_{}_{}",
                method_name, "ifend", label_ctr
            ))));
            operations.push(RelativeOperation::new(ByteCodeOp::Label(format!(
                "{}_{}_{}",
                method_name, "else", label_ctr
            ))));
            generate_function_bytecode(
                &(**els).0,
                store_ct,
                increased_labelctr,
                method_name,
                mem_store,
                operations,
            );
            operations.push(RelativeOperation::new(ByteCodeOp::Label(format!(
                "{}_{}_{}",
                method_name, "ifend", label_ctr
            ))));
        }
        Expr::Print(expr) => {
            generate_function_bytecode(
                &(**expr).0,
                store_ct,
                label_ctr,
                method_name,
                mem_store,
                operations,
            );
            operations.push(RelativeOperation::new(ByteCodeOp::Print))
        }
        Expr::Return(expr) => {
            generate_function_bytecode(
                &(**expr).0,
                store_ct,
                label_ctr,
                method_name,
                mem_store,
                operations,
            );
            operations.push(RelativeOperation::new(ByteCodeOp::Return))
        }
        Expr::Assign(ident, expression, next) => {
            generate_function_bytecode(
                &(**expression).0,
                store_ct,
                label_ctr,
                method_name,
                mem_store,
                operations,
            );
            let variable_local_ct = mem_store.get(ident);
            operations.push(RelativeOperation::new(ByteCodeOp::LocalSet(
                *variable_local_ct.unwrap(),
            )));
            store_ct += 1;
            generate_function_bytecode(
                &(**next).0,
                store_ct,
                label_ctr,
                method_name,
                mem_store,
                operations,
            );
        }
        Expr::Loop(cond, body) => {
            let increased_labelctr = label_ctr + 1;
            operations.push(RelativeOperation::new(ByteCodeOp::Label(format!(
                "{}_{}_{}",
                method_name, "loopstart", label_ctr
            ))));
            generate_function_bytecode(
                &(**cond).0,
                store_ct,
                increased_labelctr,
                method_name,
                mem_store,
                operations,
            );
            operations.push(RelativeOperation::new(ByteCodeOp::JumpFalse(format!(
                "{}_{}_{}",
                method_name, "loopend", label_ctr
            ))));
            operations.push(RelativeOperation::new(ByteCodeOp::Label(format!(
                "{}_{}_{}",
                method_name, "loopbody", label_ctr
            ))));
            generate_function_bytecode(
                &(**body).0,
                store_ct,
                increased_labelctr,
                method_name,
                mem_store,
                operations,
            );
            operations.push(RelativeOperation::new(ByteCodeOp::Label(format!(
                "{}_{}_{}",
                method_name, "le", label_ctr
            ))));
            operations.push(RelativeOperation::new(ByteCodeOp::Label(format!(
                "{}_{}_{}",
                method_name, "loopend", label_ctr
            ))));
        }
    }
}

fn generate_function_code(function: &Func, function_name: &str) -> Vec<RelativeOperation> {
    let mut operations = Vec::new();
    let mut mem_store: HashMap<String, usize> = HashMap::new();
    let label_ctr = 0;
    operations.push(RelativeOperation::new(ByteCodeOp::Label(
        function_name.to_string(),
    )));

    for (i, arg) in function.args.iter().enumerate() {
        mem_store.insert(arg.clone(), i);
    }

    generate_function_bytecode(
        &function.body.0,
        function.args.len(),
        label_ctr,
        function_name,
        &mut mem_store,
        &mut operations,
    );
    if function_name == "main" {
        operations.push(RelativeOperation::new(ByteCodeOp::End))
    }
    operations
}

pub struct Generator {
    ast: HashMap<String, Func>,
}

impl Generator {
    pub fn new(ast: HashMap<String, Func>) -> Self {
        Generator { ast }
    }
    /// Takes the bastract syntax tree stored in the Generator and prints the generated bytecode
    pub fn generate_bytecod(&self) -> Result<Vec<ByteCodeFunction>> {
        if self.ast.contains_key("main") {
            Ok(self
                .ast
                .iter()
                .map(|func_and_name| {
                    ByteCodeFunction::new(
                        func_and_name.0.clone(),
                        generate_function_code(func_and_name.1, &func_and_name.0),
                        func_and_name.1.args.len(),
                    )
                })
                .collect())
        } else {
            bail!("No main found")
        }
    }
}
