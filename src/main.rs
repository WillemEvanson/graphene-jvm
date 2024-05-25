use graphene_jvm::vm::class::parse;

fn main() {
    let Some(file_path) = std::env::args().nth(1) else {
        eprintln!("usage: graphene_jvm [path]");
        return;
    };

    let Ok(file_contents) = std::fs::read(&file_path) else {
        eprintln!("File {file_path} does not exist");
        return;
    };

    let class = parse(&file_contents).unwrap();
    println!("{class:#?}");
}
