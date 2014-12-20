extern crate rusql;

use rusql::{rusql_exec, Rusql, TableEntry, TableHeader};

fn main() {
    let mut db = Rusql::new();

    let sql_str = "CREATE TABLE Foo(Id INTEGER PRIMARY KEY, Name TEXT); \
                   INSERT INTO Foo VALUES(1, \"Bar1\"); \
                   INSERT INTO Foo VALUES(2, \"Bar2\"); \
                   INSERT INTO Foo VALUES(3, \"Bar3\"); \
                   INSERT INTO Foo VALUES(4, \"Bar4\"); \
                   SELECT * FROM Foo;";

    rusql_exec(&mut db, sql_str.to_string(), |entry: &TableEntry, header: &TableHeader| {
        for (column, def) in entry.iter().zip(header.iter()) {
            println!("{}: {}", def.name, column);
        }
    });
}
