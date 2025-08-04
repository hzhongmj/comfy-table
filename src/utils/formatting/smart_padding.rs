// Copyright (c) 2025 Arista Networks, Inc.  All rights reserved.
// Arista Networks, Inc. Confidential and Proprietary.

use crate::{Table};
use crate::style::CellAlignment;
use crate::style::ContentArrangement;
use crate::style::TableComponent;
use crate::utils::ColumnDisplayInfo;
use crate::utils::arrangement::constraint;
use crate::utils::arrangement::helper::*;

#[derive(PartialEq)]
enum DynamicPadding {
    None,
    Left,
    Right,
}

// this is similar to available_content_width in util/arrangement/dynamic.rs
fn available_width(
    table: &Table,
    infos: &Vec<&mut ColumnDisplayInfo>,
) -> usize {
    let mut width = match table.width() {
        Some(width) => width as usize,
        None => return 0,
    };
    let visible_columns = infos.len();
    let border_count = count_border_columns(table, visible_columns);
    width = width.saturating_sub(border_count);

    // Remove all column widths; note all hidden columns are already filtered
    for info in infos.iter() {
        width = width.saturating_sub(info.width().into());
    }

    width
}

fn get_column_max_widths(
    table : &Table,
    all_infos: &[ColumnDisplayInfo],
    visible_columns : usize
) -> Vec<Option<u16>> {
    let mut max_widths= Vec::with_capacity(visible_columns);
    for (column, info) in table.columns.iter().zip( all_infos.iter()) {
        if info.is_hidden {
            continue;
        }
        let max_width = constraint::max(table, &column.constraint, visible_columns);
        max_widths.push(max_width);
    }
    max_widths
}

fn is_non_whitespace(c : Option<char>) -> bool {
    match c {
        Some(c) => !c.is_whitespace(),
        None => false,
    }
}

// This checks if we can pad this column based
//
// info: ColumnDisplayInfo for the column
// max_width: the maximum width of the column (including paddings)
// exclude_alignment: cannot pad if the column has this alignment
fn can_pad_column(
    info : &ColumnDisplayInfo,
    max_width : Option<u16>,
    exclude_alignment : CellAlignment,
) -> bool {
    if info.cell_alignment.unwrap_or(CellAlignment::Left) == exclude_alignment {
        return false;
    }
    match max_width {
        // if the total width of the column is lower than max, we can pad
        Some(max_width) => info.padding.0 + info.padding.1 + info.content_width < max_width,
        None => true
    }
}

// Compare two adjacent cells and determine whether we need extra padding
fn compare_adjacent_cells(
    cell_left: &String,
    cell_right: &String,
    display_infos: &Vec<&mut ColumnDisplayInfo>,
    max_widths: &Vec<Option<u16>>,
    column_index: usize
) -> DynamicPadding {
    if is_non_whitespace(cell_left.chars().last()) &&
        is_non_whitespace(cell_right.chars().next()) {
            // the two adjacent cells have only one whitespace inbetween (border),
            // and at least one of them has a whitespace in the middle, let's add 
            // an extra space.

            // We have a choice to pad either the left column or the right.
            // To make things look a bit nicer:
            //
            // left column              right column          pad column
            // left/center aligned      *                     left
            // right aligned            left aligned          none
            // right aligned            right/center aligned  right

            // We can get the column alignment from display info, but cells can override it.
            // The code currently does not handle the cell override which isn't common.
            #[cfg(feature = "_debug")]
            println!( "smartpad: detected column {} left {:?} right {:?}", column_index,
                       cell_left, cell_right );
            if can_pad_column(display_infos[column_index],
                              max_widths[column_index],
                              CellAlignment::Right) {
                return DynamicPadding::Left;
            }
            if can_pad_column(display_infos[column_index+1],
                              max_widths[column_index+1],
                              CellAlignment::Left) {
                return DynamicPadding::Right;
            }
        }

    return DynamicPadding::None;
}

// Update all cells in a certain column to add an extra padding to either left or right
fn update_column_padding(content : &mut [Vec<Vec<String>>],
                         display_infos: &mut Vec<&mut ColumnDisplayInfo>,
                         column_index: usize,
                         pad_left: bool) {
    #[cfg(feature = "_debug")]
    println!("smartpad: update column {} padding {}", column_index,
             if pad_left { "left" } else { "right" } );
    // adjust the padding to make the header line consistent
    display_infos[column_index].content_width += 1;
    for row in content.iter_mut() {
        for sub_row in row.iter_mut() {
            if pad_left {
                sub_row[column_index] = format!(" {}", sub_row[column_index]);
            } else {
                sub_row[column_index].push(' ');
            }
        }
    }
}

// Given a column_index, check if any two cells of this and the next column
// are only separated by one whitespace, and if so, return a result telling
// which column to pad
fn compare_adjacent_columns(
    table : &Table,
    content : &[Vec<Vec<String>>],
    display_infos: &Vec<&mut ColumnDisplayInfo>,
    max_widths: &Vec<Option<u16>>,
    column_index: usize
) -> DynamicPadding {
    if !(can_pad_column(display_infos[column_index], max_widths[column_index],
                        CellAlignment::Right ) ||
         can_pad_column(display_infos[column_index+1], max_widths[column_index+1],
                        CellAlignment::Left )) {
        return DynamicPadding::None;
    }
    
    for (row_index, row) in content.iter().enumerate() {
        if row_index == 0 && table.header().is_some() {
            continue;
        }
        for sub_row in row.iter() {
            let cell_left = &sub_row[column_index];
            let cell_right = &sub_row[column_index+1];
            let padding_needed = compare_adjacent_cells(cell_left, cell_right,
                                                        display_infos, max_widths,
                                                        column_index);
            if padding_needed != DynamicPadding::None {
                return padding_needed
            }
        }
    }
    return DynamicPadding::None;
}

pub fn smart_pad_content(
    table : &Table,
    content : &mut [Vec<Vec<String>>],
    all_infos: &mut [ColumnDisplayInfo]
) {
    // only do dynamic padding with dynamic arrangement
    match &table.arrangement {
        ContentArrangement::Dynamic => (),
        _ => return,
    }

    if table.style_or_default(TableComponent::VerticalLines) != " " {
        // if we have vertical border, no need for this padding
        // this also avoids some of the test failures
        return;
    }

    if content.len() == 0 {
        return;
    }

    // calculate the max width for each column has it may disqualify some columns from padding
    let max_widths = get_column_max_widths(table, all_infos, content[0][0].len());

    // content only has visible data, while display_infos has hidden columns.
    // to be able to index into them correctly, let's just filter them.
    let mut display_infos: Vec<&mut ColumnDisplayInfo> = all_infos
        .iter_mut()
        .filter(|info| !info.is_hidden)
        .collect();

    assert_eq!(content[0][0].len(), display_infos.len());

    // get the max width of all columns as it might affect whether we can
    // add more padding
    let mut remaining_width = available_width(table, &display_infos);
    if remaining_width == 0 {
        return;
    }

    #[cfg(feature = "_debug")]
    println!( "smartpad: available width {} visible columns {}", remaining_width, display_infos.len());

    for column_index in 0..display_infos.len()-1 {
        // if there is already padding just continue
        if display_infos[column_index].padding.1 > 0 ||
            display_infos[column_index+1].padding.0 > 0 {
                continue;
            }

        match compare_adjacent_columns(table, content, &display_infos, &max_widths,
                                       column_index) {
            DynamicPadding::Left => {
                update_column_padding(content,
                                      &mut display_infos,
                                      column_index,
                                      false );
                remaining_width -= 1;
            }
            DynamicPadding::Right => {
                update_column_padding(content,
                                      &mut display_infos,
                                      column_index+1,
                                      true );
                remaining_width -= 1;
            }
            DynamicPadding::None => (),
        }

        // if no more width available just return
        if remaining_width == 0 {
            break;
        }
    }
}

