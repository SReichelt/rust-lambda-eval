use std::io::stdin;

use lambda_calculus::raw_expr::*;
use lambda_calculus_macro::raw_expr;

fn main() {
    println!(
        "Enter a lambda expression to reduce it, or an empty line to print examples and quit."
    );
    let mut buffer = String::new();
    loop {
        let read = stdin().read_line(&mut buffer).unwrap();
        if read == 0 {
            break;
        }
        let input = buffer.trim();
        if input.is_empty() {
            print_examples();
            break;
        }
        match input.parse::<RawExpr>() {
            Ok(mut expr) => {
                println!("input: {expr}");
                let mut limit = 10000;
                if expr.reduce(&mut limit) {
                    println!("reduced: {expr}");
                    if limit == 0 {
                        println!("(reduction limit reached)");
                    }
                } else {
                    println!("not reducible");
                }
            }
            Err(msg) => eprintln!("{msg}"),
        }
        buffer.clear();
    }
}

fn print_examples() {
    let mut small_limit = 100;
    let mut perf_limit = 10000000;
    let mut mem_limit = 10000;

    let id = raw_expr!(λa.a);
    println!("id: {id}");

    let cnst = raw_expr!(λc.λb.c);
    println!("const: {cnst}");

    let mut cnst_id = raw_expr!(cnst id);
    println!("const_id: {cnst_id}");

    cnst_id.reduce(&mut small_limit);
    println!("const_id reduced: {cnst_id}");

    let mut apply = raw_expr!(λd.λe.(d e));
    println!("apply: {apply}");

    apply.reduce(&mut small_limit);
    println!("apply reduced: {apply}");

    let trsp = raw_expr!(λf.λg.(g f));
    println!("transpose: {trsp}");

    let mut trsp_id = raw_expr!(trsp id);
    println!("transpose_id: {trsp_id}");

    trsp_id.reduce(&mut small_limit);
    println!("transpose_id reduced: {trsp_id}");

    let mut trsp_id_const = raw_expr!(trsp id cnst);
    println!("transpose_id_const: {trsp_id_const}");

    trsp_id_const.reduce(&mut small_limit);
    println!("transpose_id_const reduced: {trsp_id_const}");

    let omega = raw_expr!(λh.(h h));
    println!("omega: {omega}");

    let mut omega_omega = raw_expr!(omega omega);
    println!("omega_omega: {omega_omega}");

    let mut omega_omega_limit = 1;
    omega_omega.reduce(&mut omega_omega_limit);
    println!("omega_omega reduced once: {omega_omega}");

    println!("reducing omega_omega {perf_limit} times...");
    omega_omega.reduce(&mut perf_limit);

    let omega3 = raw_expr!(λi.(i i i));
    println!("omega3: {omega3}");

    let mut omega3_omega3 = raw_expr!(omega3 omega3);
    println!("omega3_omega3: {omega3_omega3}");

    let mut omega3_omega3_limit = 1;
    omega3_omega3.reduce(&mut omega3_omega3_limit);
    println!("omega3_omega3 reduced once: {omega3_omega3}");

    omega3_omega3_limit = 1;
    omega3_omega3.reduce(&mut omega3_omega3_limit);
    println!("omega3_omega3 reduced twice: {omega3_omega3}");

    omega3_omega3_limit = 8;
    omega3_omega3.reduce(&mut omega3_omega3_limit);
    println!("omega3_omega3 reduced 10 times: {omega3_omega3}");

    println!("reducing omega3_omega3 {mem_limit} times...");
    omega3_omega3.reduce(&mut mem_limit);
}
