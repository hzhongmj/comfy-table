// Copyright (c) 2025 Arista Networks, Inc.  All rights reserved.
// Arista Networks, Inc. Confidential and Proprietary.

use pretty_assertions::assert_eq;
use comfy_table::*;
use comfy_table::ColumnConstraint::*;

fn init_table(table: &mut Table,
              headers: Vec<&str>) {
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .load_preset(comfy_table::presets::NOTHING)
        .set_style(comfy_table::TableComponent::HeaderLines, '-')
        .set_style(comfy_table::TableComponent::MiddleHeaderIntersections, ' ')
        .set_header(headers)
        .set_smart_padding(true)
        .set_width(120);

    // remove padding
    for column in table.column_iter_mut() {
        column.set_padding((0,0));
    }
}

#[test]
fn login_example() {
    let mut table = Table::new();
    init_table(&mut table, vec!["  ", "Line", "User", "Idle", "Location"]);

    table
        .add_row(vec![ "", "1 con 0", "root", "07:29:04", "-" ])
        .add_row(vec![ "", "2 vty 3", "root", "09:12:23", "10.243.214.227" ])
        .add_row(vec![ "", "3 vty 5", "root", "07:09:36", "10.243.214.212" ])
        .add_row(vec![ "*", "4 vty 7", "admin", "00:00:00", "fdfd:5c41:712d:d08e:2ce2:7eff:fea5:ae5" ]);

    table.column_mut( 1 ).unwrap().set_cell_alignment(CellAlignment::Center);

    println!("{table}");

    let expected = vec![
        "     Line   User   Idle      Location                              ",
        "-- -------- ------ --------- --------------------------------------",
        "   1 con 0  root   07:29:04  -                                     ",
        "   2 vty 3  root   09:12:23  10.243.214.227                        ",
        "   3 vty 5  root   07:09:36  10.243.214.212                        ",
        "*  4 vty 7  admin  00:00:00  fdfd:5c41:712d:d08e:2ce2:7eff:fea5:ae5" ].join("\n");

    assert_eq!(table.to_string(), expected);
}

#[test]
fn basic_padding() {
    let mut table = Table::new();
    init_table(& mut table, vec!["Left", "Hidden", "Right", "Left", "Center", "Right", "Center"]);

    table
        .add_row(vec![ "left", "hidden", "right", "left","center", "right", "center" ]);

    table.column_mut(1).unwrap().set_constraint(ColumnConstraint::Hidden);
    table.column_mut(2).unwrap().set_cell_alignment(CellAlignment::Right);
    table.column_mut(4).unwrap().set_cell_alignment(CellAlignment::Center);
    table.column_mut(5).unwrap().set_cell_alignment(CellAlignment::Right);
    table.column_mut(6).unwrap().set_cell_alignment(CellAlignment::Center);

    println!("{table}");

    let expected = vec![
        "Left  Right Left  Center  Right  Center",
        "----- ----- ----- ------- ----- -------",
        "left  right left  center  right  center" ].join("\n");

    assert_eq!(table.to_string(), expected);
}

#[test]
fn max_column_width() {
    let mut table = Table::new();
    init_table(& mut table, vec!["SinglePad", "Lower", "Upper", "DoublePad", "Both", "Absolute"]);

    table
        .add_row(vec![ "singlepad", "lower", "upper", "doublepad", "both", "absolute" ] )
        .set_constraints(vec![
            UpperBoundary(Width::Fixed(10)), // doesn't matter
            LowerBoundary(Width::Fixed(5)),
            UpperBoundary(Width::Fixed(5)),
            UpperBoundary(Width::Percentage(90)), // doesn't matter
            Boundaries {
                lower: Width::Fixed(2),
                upper: Width::Fixed(4)
            },
            Absolute(Width::Fixed(8)),
        ]);

    table.column_mut(3).unwrap().set_cell_alignment(CellAlignment::Center);
    println!("{table}");

    let expected = vec![
        "SinglePad  Lower  Upper  DoublePad  Both Absolute",
        "---------- ------ ----- ----------- ---- --------",
        "singlepad  lower  upper  doublepad  both absolute" ].join("\n");

    assert_eq!(table.to_string(), expected);
}

#[test]
fn big_header() {
    let mut table = Table::new();
    init_table(& mut table, vec!["Big Header 1", "Big Header 2"]);

    table
        .add_row(vec![ "1", "a" ])
        .add_row(vec![ "2", "b" ])
        .add_row(vec![ "3", "c" ])
        .add_row(vec![ "4", "d" ]);

    println!("{table}");

    // No extra space for header
    let expected = vec![
        "Big Header 1 Big Header 2",
        "------------ ------------",
        "1            a           ",
        "2            b           ",
        "3            c           ",
        "4            d           " ].join("\n");

    assert_eq!(table.to_string(), expected);
}
