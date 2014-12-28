use table::{TableRow, TableHeader, Table};
use definitions::{ResultColumn, RusqlStatement, InsertDef, SelectDef};
use definitions::{AlterTableDef, AlterTable, Expression};
use definitions::{DeleteDef, InsertDataSource, UpdateDef};
use expressions::{ExpressionResult, ExpressionEvaluator, expr_to_literal};
use rusql::Rusql;

peg_file! parser("sql.rustpeg");

pub fn rusql_exec(db: &mut Rusql, sql_str: &str, callback: |&TableRow, &TableHeader|) -> Option<Table> {
    match parser::rusql_parse(sql_str) {
        Ok(res) => {
            for stmt in res.into_iter() {
                match stmt {
                    RusqlStatement::AlterTable(alter_table_def) => alter_table(db, alter_table_def),
                    RusqlStatement::CreateTable(table_def) => db.create_table(table_def),
                    RusqlStatement::Delete(delete_def) => delete(db, delete_def),
                    RusqlStatement::DropTable(drop_table_def) => db.drop_table(&drop_table_def.name),
                    RusqlStatement::Insert(insert_def) => insert(db, insert_def),
                    RusqlStatement::Select(select_def) => return Some(select(db, select_def, |a, b| callback(a, b))),
                    RusqlStatement::Update(update_def) => update(db, update_def),
                }
            }
        }
        Err(e) => println!("syntax error: {}", e),
    }
    None
}

fn alter_table(db: &mut Rusql, alter_table_def: AlterTableDef) {
    match alter_table_def.mode {
        AlterTable::RenameTo(new_name) => db.rename_table(&alter_table_def.name, new_name),
        AlterTable::AddColumn(column_def) => db.get_mut_table(&alter_table_def.name)
                                               .add_column(column_def),
    }
}

fn delete(db: &mut Rusql, delete_def: DeleteDef) {
    let table = db.get_mut_table(&delete_def.name);

    if let Some(ref expr) = delete_def.where_expr {
        // FIXME just making the borrow checker happy...
        let header = table.header.clone();
        table.delete_where(|row| ExpressionEvaluator::new(row, &header).eval_bool(expr));
    } else {
        table.clear();
    }
}

fn insert(db: &mut Rusql, insert_def: InsertDef) {
    match insert_def.data_source {
        InsertDataSource::Values(column_data) => {
            let mut table = db.get_mut_table(&insert_def.table_name);
            table.insert(column_data, &insert_def.column_names);
        }
        InsertDataSource::Select(select_def) => {
            let results_table = select(db, select_def, |_,_| {});
            let mut table = db.get_mut_table(&insert_def.table_name);

            for (_, row) in results_table.data.into_iter() {
                table.push_row(row);
            }
        }
        _ => {}
    }
}

fn update(db: &mut Rusql, update_def: UpdateDef) {
    let mut table = db.get_mut_table(&update_def.name);

    for (_, row) in table.data.iter_mut() {
        if let Some(ref expr) = update_def.where_expr {
            if !ExpressionEvaluator::new(row, &table.header).eval_bool(expr) {
                continue;
            }
        }

        for &(ref name, ref expr) in update_def.set.iter() {
            let x = table.header.iter().position(|ref cols| &cols.name == name).unwrap();

            row[x] = expr_to_literal(expr);
        }
    }
}

fn product(tables: Vec<&Table>, input_product: &mut Table, new_row_opt: Option<TableRow>) {
    let mut remaining = tables.clone();
    if let Some(table) = remaining.remove(0) {
        for row in table.data.values() {
            let mut new_row: TableRow = if let Some(ref new_row) = new_row_opt {
                new_row.clone()
            } else {
                Vec::new()
            };

            new_row.push_all(&*row.clone());

            product(remaining.clone(), input_product, Some(new_row));
        }
    } else {
        if let Some(new_row) = new_row_opt {
            input_product.push_row(new_row);
        }
    }
}

fn select(db: &mut Rusql, select_def: SelectDef, callback: |&TableRow, &TableHeader|) -> Table {
    let mut input_tables: Vec<&Table> = Vec::new();
    let mut input_product = generate_inputs(db, &mut input_tables, &select_def);

    filter_inputs(&mut input_product, &input_tables, &select_def);

    let results_table = generate_result_set(input_product, &input_tables, &select_def);

    for row in results_table.data.values() {
        callback(row, &results_table.header);
    }

    results_table
}

fn generate_inputs<'a>(db: &'a Rusql, input_tables: &mut Vec<&'a Table>, select_def: &SelectDef) -> Table {
    // https://www.sqlite.org/lang_select.html#fromclause
    let mut input_header: TableHeader = Vec::new();

    if let Some(ref table_or_subquery) = select_def.table_or_subquery {

        for name in table_or_subquery.iter() {
            let table = db.get_table(name);
            input_tables.push(table);
            input_header.push_all(&*table.header.clone());
        }

        let mut input_product = Table::new_result_table(input_header);

        product(input_tables.clone(), &mut input_product, None);

        input_product
    } else {
       let mut input_product = Table::new_result_table(input_header);
       let empty_row: TableRow = Vec::new();
       input_product.push_row(empty_row);

       input_product
    }
}

fn filter_inputs(input_product: &mut Table, input_tables: &Vec<&Table>, select_def: &SelectDef) {
    // https://www.sqlite.org/lang_select.html#whereclause

    if let Some(ref expr) = select_def.where_expr {
        let header = input_product.header.clone();
        input_product.delete_where(|row| {
            !ExpressionEvaluator::new(row, &header).with_tables(input_tables.clone())
                                                   .eval_bool(expr)
        });
    }
}

fn generate_result_set(input_product: Table, input_tables: &Vec<&Table>, select_def: &SelectDef) -> Table {
    // https://www.sqlite.org/lang_select.html#resultset
    let results_header: TableHeader = Vec::new();
    let mut results_table = Table::new_result_table(results_header);

    for row in input_product.data.values() {
        match select_def.result_column {
            ResultColumn::Expressions(ref exprs) => generate_row_from_expressions(&mut results_table, row, exprs, input_tables),
            ResultColumn::Asterisk => {
                if results_table.header.len() == 0 {
                    results_table.header = input_product.header.clone();
                }
                results_table.push_row(row.clone());
            }
        }
    }

    results_table
}

fn generate_row_from_expressions(results_table: &mut Table, row: &TableRow, exprs: &Vec<Expression>, input_tables: &Vec<&Table>) {
    let mut new_row: TableRow = Vec::new();
    let push_header = if results_table.header.len() == 0 { true } else { false };

    for expr in exprs.iter() {
        if push_header {
            match ExpressionEvaluator::new(row, &results_table.header).with_tables(input_tables.clone())
                                                                      .with_column_def()
                                                                      .eval_expr(expr) {
                ExpressionResult::ColumnDef(def) => results_table.header.push(def.clone()),
                _ => {}, // FIXME No idea
            }
        }
        match ExpressionEvaluator::new(row, &results_table.header).with_tables(input_tables.clone())
                                                                  .eval_expr(expr) {
            ExpressionResult::Value(v) => new_row.push(v),
            _ => {}, // FIXME No idea
        }
    }

    results_table.push_row(new_row);
}
