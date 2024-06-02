use graphene_jvm::vm::class::parse;

fn main() {
    let Some(rt_jar_path) = std::env::args().nth(1) else {
        eprintln!("usage: graphene_jvm [rt_jar_path]");
        return;
    };

    visit_dirs(std::path::Path::new(&rt_jar_path)).unwrap();
}

pub fn visit_dirs(dir: &std::path::Path) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            visit_dirs(&path)?
        }
    } else {
        println!("{}", dir.display());
        let file_contents = std::fs::read(&dir).unwrap();
        let _class = parse(&file_contents).unwrap();
    }
    Ok(())
}
