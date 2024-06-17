use std::path::PathBuf;

use graphene_jvm::string::from_utf8;
use graphene_jvm::vm::{execute, ClassManager};

fn main() {
    let (class_manager, main_class) = if std::env::args().len() < 3 {
        eprintln!("usage: graphene_jvm [class files] [main class]");
        return;
    } else {
        let file_count = std::env::args_os().len();
        let mut stack = std::env::args_os()
            .skip(1)
            .take(file_count - 2)
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        let main_class = std::env::args().last().unwrap();

        let mut class_manager = ClassManager::new();
        while let Some(entry) = stack.pop() {
            if entry.is_dir() {
                for entry in std::fs::read_dir(entry).unwrap() {
                    let entry = entry.unwrap();
                    stack.push(entry.path());
                }
            } else if let Some(extension) = entry.extension() {
                if extension == "class" {
                    let file_contents = std::fs::read(&entry).unwrap();
                    class_manager.load(&file_contents).unwrap();
                }
            }
        }
        (class_manager, main_class)
    };
    let main_class = from_utf8(main_class.as_str());
    println!("{}", main_class);

    execute(&class_manager, &main_class);
}
