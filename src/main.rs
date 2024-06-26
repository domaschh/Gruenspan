use anyhow::bail;
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::stream::Stream;
use codegen::Generator;
use runtime::Runtime;
use std::{env, fs};

use chumsky::{error::Simple, Parser};
use parser::{ast_evaluator, funcs_parser, lexer};

pub mod codegen;
pub mod parser;
pub mod runtime;

fn main() {
    let src = fs::read_to_string(env::args().nth(1).expect("Expected file argument"))
        .expect("Failed to read file");

    let (tokens, mut errs) = lexer().parse_recovery(src.as_str());

    let parse_errs = if let Some(tokens) = tokens {
        let len = src.chars().count();
        let (ast, parse_errs) =
            funcs_parser().parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

        if let Some(funcs) = ast.filter(|_| errs.len() + parse_errs.len() == 0) {
            //TODO cloning here is super expensive big nono
            let generator = Generator::new(funcs.clone());
            let bytecode = generator.generate_bytecod().unwrap();
            // // bytecode.iter().for_each(|op| {
            // //     println!("Name: {}", op.name);
            // //     println!("Arg ct: {}", op.arg_ct);
            // //     println!("Operations");
            // //     op.ops.iter().for_each(|op| println!("{:?}", op));
            // // });
            let mut runtime = Runtime::new(bytecode);
            // // println!("Execution in VM starts");
            if let Ok(result) = runtime.execute_program() {
                println!("Runtime Execution returned: {}", result);
            } else {
                panic!("Runtime Execution failed");
            }
            // This should not be in the final output this is the AST inline interpreter
            // println!("Ast interpreter starts");
            // if let Some(main) = funcs.get("main") {
            //     match ast_evaluator(&main.body, &funcs, &mut Vec::new()) {
            //         Ok(val) => println!("Return value: {}", val),
            //         Err(e) => errs.push(Simple::custom(e.span, e.msg)),
            //     }
            // } else {
            //     panic!("No main function!");
            // }
        }

        parse_errs
    } else {
        Vec::new()
    };

    errs.into_iter()
        .map(|e| e.map(|c| c.to_string()))
        .chain(parse_errs.into_iter().map(|e| e.map(|tok| tok.to_string())))
        .for_each(|e| {
            let report = Report::build(ReportKind::Error, (), e.span().start);

            let report = match e.reason() {
                chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                    .with_message(format!(
                        "Unclosed delimiter {}",
                        delimiter.fg(Color::Yellow)
                    ))
                    .with_label(
                        Label::new(span.clone())
                            .with_message(format!(
                                "Unclosed delimiter {}",
                                delimiter.fg(Color::Yellow)
                            ))
                            .with_color(Color::Yellow),
                    )
                    .with_label(
                        Label::new(e.span())
                            .with_message(format!(
                                "Must be closed before this {}",
                                e.found()
                                    .unwrap_or(&"end of file".to_string())
                                    .fg(Color::Red)
                            ))
                            .with_color(Color::Red),
                    ),
                chumsky::error::SimpleReason::Unexpected => report
                    .with_message(format!(
                        "{}, expected {}",
                        if e.found().is_some() {
                            "Unexpected token in input"
                        } else {
                            "Unexpected end of input"
                        },
                        if e.expected().len() == 0 {
                            "something else".to_string()
                        } else {
                            e.expected()
                                .map(|expected| match expected {
                                    Some(expected) => expected.to_string(),
                                    None => "end of input".to_string(),
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    ))
                    .with_label(
                        Label::new(e.span())
                            .with_message(format!(
                                "Unexpected token {}",
                                e.found()
                                    .unwrap_or(&"end of file".to_string())
                                    .fg(Color::Red)
                            ))
                            .with_color(Color::Red),
                    ),
                chumsky::error::SimpleReason::Custom(msg) => report.with_message(msg).with_label(
                    Label::new(e.span())
                        .with_message(format!("{}", msg.fg(Color::Red)))
                        .with_color(Color::Red),
                ),
            };

            report.finish().print(Source::from(&src)).unwrap();
        });
}
