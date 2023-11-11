use clap::Parser;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(short, long, default_value = "show")]
    action: String,

    #[arg(short, long, default_value = "")]
    item: String,

    #[arg(short, long, default_value = "999")]
    position: usize,
}

#[derive(Clone, Serialize, Deserialize)]
struct ToDo {
    item: String,
    done: bool,
    position: usize,
}

#[derive(Clone, Serialize, Deserialize)]
struct ToDoProps {
    done: bool,
    position: usize,
}

struct ToDos {
    todos: Vec<ToDo>,
}

impl ToDos {
    fn insert(&mut self, item: String) {
        let size = self.todos.len();
        self.todos.push(ToDo {
            item,
            done: false,
            position: size,
        });
    }

    fn remove(&mut self, position: usize) {
        _ = self.todos.remove(position);
        let num = position;
        for i in num..self.todos.len() {
            let todo_opt = self.todos.get_mut(i);
            match todo_opt {
                Some(todo) => todo.position -= 1,
                None => eprintln!("Unknown index"),
            }
        }
    }

    fn update(&mut self, position: usize) {
        let todo_opt = self.todos.get_mut(position);
        match todo_opt {
            Some(todo) => {
                todo.done = !todo.done;
                println!("{}", todo.done);
            }
            None => {
                eprintln!("No todo found to update. Given index: {}", position);
            }
        }
    }
    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        match fs::remove_file("db.json") {
            Ok(_) => (),
            Err(err) => eprintln!("Old file could not be deleted. Err: {}", err),
        }
        let f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open("db.json")?;
        serde_json::to_writer_pretty(f, &self.todos)?;
        Ok(())
    }
    fn new() -> Result<ToDos, std::io::Error> {
        let f = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open("db.json")?;

        match serde_json::from_reader(f) {
            Ok(todos) => Ok(ToDos { todos }),
            Err(e) if e.is_eof() => Ok(ToDos { todos: vec![] }),
            Err(e) => panic!("Error {}", e),
        }
    }
}

impl fmt::Display for ToDos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        _ = write!(f, "Todos: \n\n");
        _ = write!(f, "   Done |  ToDo\n");
        let todos: &Vec<_> = &self.todos;
        for todo in todos {
            let done = if todo.done { "[x]" } else { "[ ]" };
            _ = write!(f, "{}. {}  |  {}\n", todo.position + 1, done, todo.item);
        }
        write!(f, "")
    }
}

fn main() {
    let args = Args::parse();
    let mut todos = ToDos::new().expect("Initialisierung fehlgeschlagen");
    if args.action == "insert" {
        todos.insert(args.item);
        save_todos(&todos);
    } else if args.action == "update" {
        match convert_position_to_index(args.position, todos.todos.len()) {
            Ok(index) => {
                todos.update(index);
                save_todos(&todos);
            }
            Err(err) => eprintln!("Invalid Index {}", err),
        }
    } else if args.action == "remove" {
        match convert_position_to_index(args.position, todos.todos.len()) {
            Ok(index) => {
                todos.remove(index);
                save_todos(&todos);
            }
            Err(err) => eprintln!("Invalid Index {}", err),
        }
    } else if args.action == "show" {
        println!("{}", todos)
    }
}

fn save_todos(todos: &ToDos) {
    match todos.save() {
        Ok(_) => println!("{}", todos),
        Err(err) => eprintln!("Saving gone wrong. Err: {:?}", err),
    }
}

fn convert_position_to_index(position: usize, list_size: usize) -> Result<usize, usize> {
    match position < 1 || position > list_size {
        false => Ok(position - 1),
        true => Err(position),
    }
}
