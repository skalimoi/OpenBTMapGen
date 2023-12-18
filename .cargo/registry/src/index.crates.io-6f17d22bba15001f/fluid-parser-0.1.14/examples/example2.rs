use fluid_parser::lexer::Lexer;
use fluid_parser::parser::Parser;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("Expected a file input!");
    }
    let s = std::fs::read_to_string(&args[1]).expect("Failed to read file!");
    let l = Lexer::new(&s);
    let mut p = Parser::new(l);
    let a = p.parse();
    println!("{:#?}", a);
}
