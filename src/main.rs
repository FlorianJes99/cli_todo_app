use core::fmt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
};

#[derive(Debug)]
struct NewArgs {
    action: Action,
    suffix: String,
}

#[derive(Debug)]
enum Action {
    Add,
    Update,
    Remove,
    Exit,
    Help,
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
        _ = write!(f, "Nr Done |  ToDo\n");
        let todos: &Vec<_> = &self.todos;
        for todo in todos {
            let done = if todo.done { "[x]" } else { "[ ]" };
            _ = write!(f, "{}. {}  |  {}\n", todo.position + 1, done, todo.item);
        }
        write!(f, "")
    }
}

fn main() {
    let mut todos = ToDos::new().expect("Initialisierung fehlgeschlagen");
    println!("{}", todos);
    loop {
        let input = prompt("> ");
        let result = parse_input(&input);
        if result.is_err() {
            println!("{}\n{}", &result.unwrap_err(), get_help_string());
            continue;
        }
        println!();
        let args = result.unwrap();
        if handle_args(args, &mut todos) {
            break;
        }
    }
}

fn handle_args(args: NewArgs, todos: &mut ToDos) -> bool {
    match args.action {
        Action::Add => {
            todos.insert(args.suffix);
            save_todos(&todos);
            false
        }
        Action::Update => match convert_position_to_index(&args.suffix, todos.todos.len()) {
            Ok(index) => {
                todos.update(index);
                save_todos(&todos);
                false
            }
            Err(err) => {
                eprintln!("Invalid Index {}", err);
                false
            }
        },
        Action::Remove => match convert_position_to_index(&args.suffix, todos.todos.len()) {
            Ok(index) => {
                todos.remove(index);
                save_todos(&todos);
                false
            }
            Err(err) => {
                eprintln!("Invalid Index {}", err);
                false
            }
        },
        Action::Help => {
            println!("{}", get_help_string());
            false
        }
        Action::Exit => true,
    }
}

fn get_help_string() -> &'static str {
    "This are valid inputs:\n add \"todo\", update \"number\", remove \"number\" , exit or help"
}

fn save_todos(todos: &ToDos) {
    match todos.save() {
        Ok(_) => println!("{}", todos),
        Err(err) => eprintln!("Saving gone wrong. Err: {:?}", err),
    }
}

fn convert_position_to_index(position: &str, list_size: usize) -> Result<usize, &'static str> {
    match position.parse::<usize>() {
        Err(_) => Err("Not a valid Number"),
        Ok(position) => match position < 1 || position > list_size {
            false => Ok(position - 1),
            true => Err("Not a valid index"),
        },
    }
}

fn parse_input(line: &str) -> Result<NewArgs, &'static str> {
    let mut iter = line.splitn(2, " ");
    let action_str = iter.next();
    if action_str.is_none() {
        return Err("No Action defined");
    }
    let action = get_action(action_str.unwrap());
    if action.is_err() {
        return Err(action.unwrap_err());
    }
    match iter.next() {
        Some(suffix) => Ok(NewArgs {
            action: action.unwrap(),
            suffix: suffix.to_string(),
        }),
        None => Ok(NewArgs {
            action: action.unwrap(),
            suffix: "".to_string(),
        }),
    }
}

fn get_action(action_str: &str) -> Result<Action, &'static str> {
    if action_str == "add" {
        Ok(Action::Add)
    } else if action_str == "update" {
        Ok(Action::Update)
    } else if action_str == "remove" {
        Ok(Action::Remove)
    } else if action_str == "exit" {
        Ok(Action::Exit)
    } else if action_str == "help" {
        Ok(Action::Help)
    } else {
        Err("Invalid Action!")
    }
}

fn prompt(name: &str) -> String {
    let mut line = String::new();
    print!("{}", name);
    std::io::stdout()
        .flush()
        .expect("Could not write to terminal");
    std::io::stdin()
        .read_line(&mut line)
        .expect("Could not read input!");
    line.trim().to_string()
}
