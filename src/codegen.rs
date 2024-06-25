use crate::parser::{BinaryOp, Expr, Func, Value};
use anyhow::{bail, Result};
use std::collections::HashMap;

#[derive(Debug)]
pub struct RelativeOperation {
    bop_type: ByteCodeOp,
}

impl RelativeOperation {
    fn new(bop_type: ByteCodeOp) -> Self {
        RelativeOperation { bop_type }
    }
}

#[derive(Debug)]
enum BopVal {
    Number(f64),
    Boolean(bool),
    String(String),
    List(Vec<BopVal>),
}

impl From<&Value> for BopVal {
    fn from(value: &Value) -> Self {
        match value {
            Value::Null => panic!("Wtf converstion from &Val to BopVal failed"),
            Value::Bool(b) => BopVal::Boolean(*b),
            Value::Num(n) => BopVal::Number(*n),
            Value::Str(sr) => BopVal::String(sr.clone()),
            Value::List(l) => BopVal::List(l.into_iter().map(|a| a.into()).collect()),
            Value::Func(fp) => panic!("Wtf converstion from &Val to BopVal failed"),
        }
    }
}

#[derive(Debug)]
enum ByteCodeOp {
    Return,
    LocalGet(usize),
    LocalSet(usize),
    Load(usize),
    Const(BopVal),
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
    JumpTrue,
    JumpFalse,
    End,
}

#[derive(Debug)]
pub struct BFunc {
    pub name: String,
    pub ops: Vec<RelativeOperation>,
    pub arg_ct: usize,
}

impl BFunc {
    fn new(name: String, ops: Vec<RelativeOperation>, arg_ct: usize) -> Self {
        BFunc { name, ops, arg_ct }
    }
}

fn generate_function_bytecode(
    expr: &Expr,
    mut store_ct: usize,
    mem_store: &mut HashMap<String, usize>,
    operations: &mut Vec<RelativeOperation>,
) {
    match expr {
        Expr::Error => unreachable!(),
        Expr::Value(val) => match val {
            Value::Null => {}
            Value::Bool(bool) => operations.push(RelativeOperation::new(ByteCodeOp::Const(
                BopVal::Boolean(*bool),
            ))),
            Value::Num(num) => operations.push(RelativeOperation::new(ByteCodeOp::Const(
                BopVal::Number(*num),
            ))),
            Value::Str(str) => operations.push(RelativeOperation::new(ByteCodeOp::Const(
                BopVal::String(str.clone()),
            ))),
            Value::List(list) => operations.push(RelativeOperation::new(ByteCodeOp::Const(
                BopVal::List(list.into_iter().map(|val| val.into()).collect()),
            ))),
            Value::Func(fp) => println!("When am I called {:?}", fp),
        },
        Expr::List(_) => todo!(),
        Expr::LocalVar(varname) => operations.push(RelativeOperation::new(ByteCodeOp::LocalGet(
            *mem_store.get(varname).unwrap(),
        ))),
        Expr::Let(variable, expression, other) => {
            generate_function_bytecode(&(**expression).0, store_ct, mem_store, operations);
            mem_store.insert(variable.clone(), store_ct);
            operations.push(RelativeOperation::new(ByteCodeOp::LocalSet(store_ct)));
            store_ct += 1;
            generate_function_bytecode(&(**other).0, store_ct, mem_store, operations);
        }
        Expr::Then(this_expr, next_expr) => {
            generate_function_bytecode(&(**this_expr).0, store_ct, mem_store, operations);
            generate_function_bytecode(&(**next_expr).0, store_ct, mem_store, operations);
        }
        Expr::Binary(lhs, operation, rhs) => {
            generate_function_bytecode(&(**lhs).0, store_ct, mem_store, operations);
            generate_function_bytecode(&(**rhs).0, store_ct, mem_store, operations);
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
                generate_function_bytecode(&arg.0, store_ct, mem_store, operations);
            }
            let Expr::LocalVar(funcname_vale) = &func_name.0 else {
                panic!("Funcname not string");
            };

            operations.push(RelativeOperation::new(ByteCodeOp::Call(
                funcname_vale.clone(),
                arguments.0.len(),
            )));
        }
        Expr::If(a, b, c) => todo!(),
        Expr::Print(expr) => {
            generate_function_bytecode(&(**expr).0, store_ct, mem_store, operations);
            operations.push(RelativeOperation::new(ByteCodeOp::Print))
        }
        Expr::Return(expr) => {
            generate_function_bytecode(&(**expr).0, store_ct, mem_store, operations);
            operations.push(RelativeOperation::new(ByteCodeOp::Return))
        }
    }
}

fn generate_function_code(function: &Func) -> Vec<RelativeOperation> {
    let mut operations = Vec::new();
    let mut mem_store: HashMap<String, usize> = HashMap::new();
    let mut mem_counter = 0;

    //Generate Local.Get n for the parameters
    for (i, arg) in function.args.iter().enumerate() {
        operations.push(RelativeOperation::new(ByteCodeOp::LocalGet(i)));
        mem_store.insert(arg.clone(), mem_counter);
        mem_counter += 1;
    }
    //the into call recursively constructs the bytecode

    println!("Memcount after argumenst {mem_counter}");
    generate_function_bytecode(
        &function.body.0,
        mem_counter,
        &mut mem_store,
        &mut operations,
    );
    operations
}
impl From<&Func> for Vec<RelativeOperation> {
    fn from(function: &Func) -> Self {
        let mut operations = Vec::new();
        let mut mem_store: HashMap<String, usize> = HashMap::new();
        let mut mem_counter = 0;

        //Generate Local.Get n for the parameters
        for (i, arg) in function.args.iter().enumerate() {
            operations.push(RelativeOperation::new(ByteCodeOp::LocalGet(i)));
            mem_store.insert(arg.clone(), mem_counter);
            mem_counter += 1;
        }
        //the into call recursively constructs the bytecode

        println!("Memcount after argumenst {mem_counter}");
        generate_function_bytecode(
            &function.body.0,
            mem_counter,
            &mut mem_store,
            &mut operations,
        );
        operations
    }
}

pub struct Generator {
    ast: HashMap<String, Func>,
}

impl Generator {
    pub fn new(ast: HashMap<String, Func>) -> Self {
        Generator { ast }
    }

    /// Takes the bastract syntax tree stored in the Generator and prints the generated bytecode
    pub fn generate_bytecod(&self) -> Result<Vec<BFunc>> {
        if self.ast.contains_key("main") {
            Ok(self
                .ast
                .iter()
                .map(|func_and_name| {
                    BFunc::new(
                        func_and_name.0.clone(),
                        func_and_name.1.into(),
                        func_and_name.1.args.len(),
                    )
                })
                .collect())
        } else {
            bail!("No main found")
        }
    }
}
