use definitions::{TableDef, LiteralValue, ColumnDef, ColumnConstraint};

use std::cell::Cell;
use std::cmp::max;
use std::collections::BTreeMap;
use std::fmt;
use std::iter::repeat;

pub type TableRow = Vec<LiteralValue>;
pub type TableHeader = Vec<ColumnDef>;
pub type PkType = usize;

pub struct RowFormat<'a>(pub &'a TableRow);
pub struct HeaderFormat<'a>(pub &'a TableHeader);

#[derive(PartialEq)]
pub struct Table {
    pub name: String,
    pub header: TableHeader,
    pub data: BTreeMap<PkType, TableRow>,
    pub pk: Option<PkType>,
    pub max_pk: Cell<PkType>,
}

impl Table {
    pub fn new(table_def: TableDef) -> Table {
        let mut table = Table {
            name: table_def.table_name,
            header: table_def.columns,
            data: BTreeMap::new(),
            pk: None,
            max_pk: Cell::new(0),
        };
        table.process_constraints();

        table
    }

    pub fn new_result_table(header: TableHeader) -> Table {
        Table {
            name: "".to_string(),
            header: header,
            data: BTreeMap::new(),
            pk: None,
            max_pk: Cell::new(0),
        }
    }
    pub fn get_column_def_by_name(&self, name: &String) -> Option<&ColumnDef> {
        self.header.iter().find(|&cols| &cols.name == name)
    }

    pub fn get_column_index(&self, name: &String) -> Option<usize> {
        self.header.iter().position(|ref cols| &cols.name == name)
    }

    pub fn has_row(&self, pk: PkType) -> bool {
        self.data.contains_key(&pk)
    }

    pub fn assert_size(&self) {
        let header_size = self.header.len();

        for row in self.data.values() {
            assert!(row.len() == header_size);
        }
    }

    pub fn add_column(&mut self, column_def: ColumnDef) {
        self.header.push(column_def);

        for (_, row) in self.data.iter_mut() {
            row.push(LiteralValue::Null);
        }
    }

    pub fn add_columns(&mut self, column_defs: Vec<ColumnDef>) {
        for def in column_defs.into_iter() {
            self.add_column(def);
        }
    }

    pub fn insert(&mut self, column_data: Vec<TableRow>,
                  specified_columns: &Option<Vec<String>>) {
        for column_data in column_data.into_iter() {
            if let &Some(ref column_names) = specified_columns {
                assert!(column_names.len() == column_data.len());
                let mut row: TableRow = repeat(LiteralValue::Null).take(self.header.len()).collect();

                for (name, data) in column_names.iter().zip(column_data.into_iter()) {
                    row[self.get_column_index(name).unwrap()] = data;
                }

                if let Some(i) = self.pk {
                    if row[i] == LiteralValue::Null {
                        row[i] = LiteralValue::Integer((self.max_pk.get() + 1) as isize);
                    }
                }

                self.push_row(row);
            } else {
                self.push_row(column_data);
            }
        }
    }

    pub fn push_row(&mut self, row: TableRow) {
        if let Some(i) = self.pk {
            let pk = row[i].clone().to_uint();

            self.max_pk.set(max(self.max_pk.get(), pk));
            self.data.insert(pk, row);
        } else {
            self.max_pk.set(self.max_pk.get() + 1);
            self.data.insert(self.max_pk.get(), row);
        }
    }

    pub fn delete_where<F: Fn(&TableRow) -> bool>(&mut self, f: F) {
        let mut keys: Vec<PkType> = Vec::new();

        for (key, row) in self.data.iter() {
            if !f(row) {
                continue;
            }
            keys.push(key.clone());
        }

        for key in keys.iter() {
            self.data.remove(key);
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn process_constraints(&mut self) {
        for (i, column) in self.header.iter().enumerate() {
            for constraint in column.column_constraints.iter() {
                match constraint {
                    &ColumnConstraint::PrimaryKey => self.pk = Some(i),
                }
            }
        }
    }
}

impl<'a> fmt::String for RowFormat<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for column in self.0.iter() {
            write!(f, "{} | ", column).ok();
        }
        Ok(())
    }
}

impl<'a> fmt::String for HeaderFormat<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for def in self.0.iter() {
            write!(f, "{} | ", def.name).ok();
        }
        Ok(())
    }
}

impl fmt::String for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.header.len() > 0 {
            writeln!(f, "{}", HeaderFormat(&self.header)).ok();
        }
        for row in self.data.values() {
            writeln!(f, "{}", RowFormat(row)).ok();
        }
        Ok(())
    }
}

pub fn get_column(name: &String, row: &TableRow, head: &TableHeader, offset: Option<usize>) -> LiteralValue {
    let x = if let Some(x) = offset { x } else { 0 };
    row[head.iter().position(|ref def| def.name == *name).unwrap() + x].clone()
}
