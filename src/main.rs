#![allow(unused_variables)]

use std::env;
use std::fs;

pub mod compiler;
use compiler::{expander, parser, reducer, tokenizer};
use parser::Ast;
use tokenizer::Token;

// use notify::{raw_watcher, RawEvent, RecursiveMode, Watcher};
// use std::sync::mpsc::channel;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() < 1 {
        eprintln!("Usage: rpl.exe <source>");
        std::process::exit(1);
    }

    // read_file(&args[0]); // first run
    // compiler(&args[0], &args[1]);
    let source = &args[0];
    event_router(notify::op::WRITE, source);

    std::process::exit(0);

    // // Create a channel to receive the events.
    // let (tx, rx) = channel();

    // // Create a watcher object, delivering raw events.
    // // The notification back-end is selected based on the platform.
    // let mut watcher = raw_watcher(tx).unwrap();

    // // Add a path to be watched. All files and directories at that path and
    // // below will be monitored for changes.
    // watcher.watch(source, RecursiveMode::Recursive).unwrap();

    // loop {
    // 	match rx.recv() {
    // 		Ok(RawEvent {
    // 			path: Some(path),
    // 			op: Ok(op),
    // 			cookie,
    // 		}) => {
    // 			println!("{:?} {:?} ({:?})", op, path, cookie);
    // 			event_router(op, source, target);
    // 		}
    // 		Ok(event) => println!("broken event: {:?}", event),
    // 		Err(e) => println!("watch error: {:?}", e),
    // 	}
    // }
}

fn event_router(operation: notify::Op, source: &String) {
    match operation {
        notify::op::WRITE => {
            let msg = match compile(source) {
                Ok(msg) => msg,
                Err(msg) => msg,
            };
            let error_path = &mut source.clone();
            error_path.push_str(&".errors".to_string());
            write_file(error_path, &msg);
        }
        _ => {}
    };
}

fn compile(source: &String) -> Result<String, String> {
    let input = read_file(source);
    //
    //
    let tokens = tokenizer::tokenizer(&input)?;
    let token_path = &mut source.clone();
    token_path.push_str(&".tokens".to_string());
    write_file(token_path, &token_string(&tokens));
    //
    //
    let parse = parser::parser(&tokens)?;
    let parse_path = &mut source.clone();
    parse_path.push_str(&".ast".to_string());
    write_file(parse_path, &ast_string(&parse));

    //
    //
    let (expand, maps) = expander::expander(&parse)?;
    let expand_path = &mut source.clone();
    expand_path.push_str(&".expand".to_string());
    write_file(expand_path, &format!("{:#?}", &expand));

    let maps_path = &mut source.clone();
    maps_path.push_str(&".maps".to_string());
    write_file(maps_path, &format!("{:#?}", &maps));

    //
    //
    // let (typed, map) = typer::typer(&expand, map)?;
    // let typed_path = &mut source.clone();
    // typed_path.push_str(&".typed".to_string());
    // write_file(typed_path, &format!("{:#?}", &typed));

    let _ = reducer::reducer(&expand, &maps)?;
    // let typed_path = &mut source.clone();
    // typed_path.push_str(&".typed".to_string());
    // write_file(typed_path, &format!("{:#?}", &typed));

    Ok("no errors".to_string())
}

fn read_file(path: &String) -> String {
    match fs::read_to_string(path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "Error: failed to read from the file '{}': {:?}",
                path, e
            );
            std::process::exit(1);
        }
    }
}

fn write_file(path: &String, data: &String) {
    match fs::write(path, data) {
        Ok(_v) => {
            // dbg!(v);
        }
        Err(e) => {
            eprintln!(
                "Error: failed to write to file '{}': {:?}",
                path, e
            );
            std::process::exit(1);
        }
    };
}

fn token_string(data: &Vec<Token>) -> String {
    let mut output = String::new();
    for group in data {
        output.push_str(
            &format!(
                "{:<12} {:<12} {:<4} {:?}\n",
                format!("{:?}", group.of.kind),
                format!("{:?}", group.of.name),
                group.meta.line,
                group.meta.text,
            )[..],
        )
    }
    output
}

fn ast_string(data: &Ast) -> String {
    format!("{:#?}", data)
}
