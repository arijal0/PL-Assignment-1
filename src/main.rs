use sexp::*;
use sexp::Atom::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;

// 1) AST for Adder
#[derive(Debug)]
enum Expr {
    Num(i32),
    Add1(Box<Expr>),
    Sub1(Box<Expr>),
    Negate(Box<Expr>),
}

// 2) Parser: Sexp -> Expr
fn parse_expr(s: &Sexp) -> Expr {
    match s {
        Sexp::Atom(I(n)) => {
            // The language is defined as 32-bit signed integers
            let v = i32::try_from(*n).expect("Number out of i32 range");
            Expr::Num(v)
        }
        Sexp::List(vec) => match &vec[..] {
            [Sexp::Atom(S(op)), e] if op == "add1" => Expr::Add1(Box::new(parse_expr(e))),
            [Sexp::Atom(S(op)), e] if op == "sub1" => Expr::Sub1(Box::new(parse_expr(e))),
            [Sexp::Atom(S(op)), e] if op == "negate" => Expr::Negate(Box::new(parse_expr(e))),
            _ => panic!("Invalid expression form: {s:?}"),
        },
        _ => panic!("Invalid expression: {s:?}"),
    }
}

// 3) Code generator: Expr -> NASM assembly (result in rax)
fn compile_expr(e: &Expr) -> String {
    match e {
        Expr::Num(n) => format!("mov rax, {}", *n),
        Expr::Add1(sub) => format!("{}\nadd rax, 1", compile_expr(sub)),
        Expr::Sub1(sub) => format!("{}\nsub rax, 1", compile_expr(sub)),
        Expr::Negate(sub) => format!("{}\nneg rax", compile_expr(sub)),
    }
}

fn main() -> std::io::Result<()> {
    // The usage of the program: cargo run -- <input.snek> <output.s>
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.snek> <output.s>", args[0]);
        std::process::exit(1);
    }
    let in_name = &args[1];
    let out_name = &args[2];

    // Reading the input program
    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;

    // Parsing the s-expression text -> AST
    let sexp = parse(&in_contents).expect("Failed to parse S-expression");
    let expr = parse_expr(&sexp);

    // Compiling the AST -> assembly body
    let body = compile_expr(&expr);

    // Wrapping as a full NASM file with an exported symbol
    let asm_program = format!(
        "section .text
global our_code_starts_here
our_code_starts_here:
  {}
  ret
",
        // indenting the body a bit so the output is readable
        body.lines()
            .map(|line| format!("  {}", line))
            .collect::<Vec<_>>()
            .join("\n")
    );

    // Writing the output assembly
    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}