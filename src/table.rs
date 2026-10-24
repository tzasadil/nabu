pub enum Row {
    Cells(Vec<String>),
    /// `first` joins column 0. `rest` follows unpadded and never widens a column,
    /// so one long error message does not push the other rows' cells away.
    Tail {
        first: String,
        rest: String,
    },
}

/// Pads every column to its widest cell, two spaces between columns.
/// A blank cell is an absent optional value. Trailing whitespace is trimmed per line.
pub fn render(rows: &[Row], indent: &str) -> String {
    let mut widths: Vec<usize> = vec![0];
    let mut widen = |column: usize, cell: &str| {
        if widths.len() <= column {
            widths.resize(column + 1, 0);
        }
        widths[column] = widths[column].max(cell.chars().count());
    };
    for row in rows {
        match row {
            Row::Cells(cells) => cells
                .iter()
                .enumerate()
                .for_each(|(column, cell)| widen(column, cell)),
            Row::Tail { first, .. } => widen(0, first),
        }
    }
    rows.iter()
        .map(|row| {
            let line = match row {
                Row::Cells(cells) => cells
                    .iter()
                    .zip(&widths)
                    .map(|(cell, width)| format!("{cell:<width$}"))
                    .collect::<Vec<_>>()
                    .join("  "),
                Row::Tail { first, rest } => format!("{first:<width$}  {rest}", width = widths[0]),
            };
            format!("{indent}{line}").trim_end().to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(rows: &[&[&str]]) -> Vec<Row> {
        rows.iter()
            .map(|row| Row::Cells(row.iter().map(|cell| cell.to_string()).collect()))
            .collect()
    }

    #[test]
    fn pads_to_the_widest_cell_and_trims_trailing_space() {
        let table = rows(&[
            &["blueprint", "ready"],
            &["backlog", "blocked", "needs blueprint"],
        ]);
        assert_eq!(
            render(&table, "  "),
            "  blueprint  ready\n  backlog    blocked  needs blueprint"
        );
    }

    #[test]
    fn blank_middle_cell_keeps_its_column() {
        let table = rows(&[
            &["a", "7/7 tasks", "", "ready"],
            &["b", "18/18 tasks", "2 slices", ""],
        ]);
        // 2 padding + 2 separator + 8 blank column + 2 separator.
        let gap = " ".repeat(14);
        assert_eq!(
            render(&table, ""),
            format!("a  7/7 tasks{gap}ready\nb  18/18 tasks  2 slices")
        );
    }

    #[test]
    fn tail_row_pads_only_its_first_cell_and_never_widens_other_columns() {
        let mut table = rows(&[&["a", "7/7 tasks", "", "ready"]]);
        table.push(Row::Tail {
            first: "long-name".into(),
            rest: "error  a very long cause".into(),
        });
        assert_eq!(
            render(&table, ""),
            "a          7/7 tasks    ready\nlong-name  error  a very long cause"
        );
    }
}
