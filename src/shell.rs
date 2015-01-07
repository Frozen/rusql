extern crate rusql;
#[cfg(not(feature = "no_readline"))]
extern crate readline;

#[cfg(not(feature = "no_readline"))]
use readline::readline;
use rusql::{rusql_exec, Rusql};

#[cfg(feature = "no_readline")]
use std::io;

#[cfg(feature = "no_readline")]
fn readline(prompt: &str) -> Option<String> {
    print!("{}", prompt);

    io::stdin().read_line().ok()
}

pub fn main() {
    let mut db = Rusql::new();
    loop {
        let mut input = readline("rusql> ").unwrap();

        while !input.as_slice().trim_right().ends_with(";")
                && !input.as_slice().trim_left().starts_with(".") {

            let continuation = readline("  ...> ").unwrap();

            input.push(' ');
            input.push_str(continuation.as_slice());
        }

        match input.as_slice() {
            ".make_foo" => {
                rusql_exec(&mut db, "CREATE TABLE Foo(Id INTEGER, Name TEXT);
                                     INSERT INTO Foo VALUES
                                            (1, \"Foo1\"), (2, \"Foo2\"), (3, \"Foo3\");
                                     CREATE TABLE Qux(QuxId INTEGER PRIMARY KEY, Nick TEXT);
                                     INSERT INTO Qux(Nick) VALUES
                                            (\"Bar1\"), (\"Bar2\"), (\"Bar3\");",
                           |_, _| ());
            }
            _ => {
                if let Some(results) = rusql_exec(&mut db, input.as_slice(), |_, _| {}) {
                    print!("{}", results);
                }
            }
        }
    }
}
