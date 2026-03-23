//! Terminal grid - 2D cell buffer with scrollback.

use std::collections::VecDeque;

use compact_str::CompactString;
use serde::Serialize;

use super::AttrFlags;

/// A single cell in the terminal grid.
#[derive(Debug, Clone)]
pub struct Cell {
    pub content: CompactString,
    pub attrs: CellAttrs,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            content: CompactString::from(" "),
            attrs: CellAttrs::default(),
        }
    }
}

impl Cell {
    pub fn reset(&mut self) {
        self.content = CompactString::from(" ");
        self.attrs = CellAttrs::default();
    }

    pub fn set(&mut self, ch: char, attrs: &CellAttrs) {
        self.content = CompactString::from(ch.to_string());
        self.attrs = attrs.clone();
    }

    pub fn clear_with_bg(&mut self, bg: &Color) {
        self.content = CompactString::from(" ");
        self.attrs = CellAttrs {
            fg: Color::Default,
            bg: bg.clone(),
            flags: AttrFlags::empty(),
        };
    }
}

/// Visual attributes for a cell.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CellAttrs {
    pub fg: Color,
    pub bg: Color,
    pub flags: AttrFlags,
}

/// Terminal color representation.
#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub enum Color {
    #[default]
    Default,
    Named(NamedColor),
    Indexed(u8),
    Rgb(u8, u8, u8),
}

/// Standard 16 ANSI colors.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum NamedColor {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
    BrightBlack = 8,
    BrightRed = 9,
    BrightGreen = 10,
    BrightYellow = 11,
    BrightBlue = 12,
    BrightMagenta = 13,
    BrightCyan = 14,
    BrightWhite = 15,
}

/// The terminal grid: visible area + scrollback buffer.
pub struct Grid {
    /// Visible cells: flat array indexed as [row * cols + col].
    cells: Vec<Cell>,
    /// Scrollback buffer.
    scrollback: VecDeque<Vec<Cell>>,
    /// Maximum scrollback lines.
    max_scrollback: usize,
    /// Grid dimensions.
    pub cols: usize,
    pub rows: usize,
    /// Scroll region (top, bottom) - inclusive, 0-indexed.
    pub scroll_top: usize,
    pub scroll_bottom: usize,
}

impl Grid {
    pub fn new(cols: usize, rows: usize) -> Self {
        let cells = vec![Cell::default(); cols * rows];
        Self {
            cells,
            scrollback: VecDeque::new(),
            max_scrollback: 1000,
            cols,
            rows,
            scroll_top: 0,
            scroll_bottom: rows - 1,
        }
    }

    /// Get a reference to a cell.
    pub fn cell(&self, row: usize, col: usize) -> &Cell {
        &self.cells[row * self.cols + col]
    }

    /// Get a mutable reference to a cell.
    pub fn cell_mut(&mut self, row: usize, col: usize) -> &mut Cell {
        &mut self.cells[row * self.cols + col]
    }

    /// Get an entire row as a slice.
    pub fn row(&self, row: usize) -> &[Cell] {
        let start = row * self.cols;
        &self.cells[start..start + self.cols]
    }

    /// Scroll the scroll region up by n lines.
    pub fn scroll_up(&mut self, n: usize) {
        for _ in 0..n {
            // Save the top line to scrollback if we're scrolling the full screen
            if self.scroll_top == 0 {
                let row: Vec<Cell> = self.row(0).to_vec();
                self.scrollback.push_back(row);
                if self.scrollback.len() > self.max_scrollback {
                    self.scrollback.pop_front();
                }
            }

            // Shift rows up within scroll region
            for row in self.scroll_top..self.scroll_bottom {
                let src_start = (row + 1) * self.cols;
                let dst_start = row * self.cols;
                for col in 0..self.cols {
                    self.cells[dst_start + col] = self.cells[src_start + col].clone();
                }
            }

            // Clear the bottom row of scroll region
            let bottom_start = self.scroll_bottom * self.cols;
            for col in 0..self.cols {
                self.cells[bottom_start + col] = Cell::default();
            }
        }
    }

    /// Scroll the scroll region down by n lines.
    pub fn scroll_down(&mut self, n: usize) {
        for _ in 0..n {
            // Shift rows down within scroll region
            for row in (self.scroll_top + 1..=self.scroll_bottom).rev() {
                let src_start = (row - 1) * self.cols;
                let dst_start = row * self.cols;
                for col in 0..self.cols {
                    self.cells[dst_start + col] = self.cells[src_start + col].clone();
                }
            }

            // Clear the top row of scroll region
            let top_start = self.scroll_top * self.cols;
            for col in 0..self.cols {
                self.cells[top_start + col] = Cell::default();
            }
        }
    }

    /// Erase from cursor to end of display.
    pub fn erase_below(&mut self, row: usize, col: usize) {
        // Erase from cursor to end of current row
        for c in col..self.cols {
            self.cell_mut(row, c).reset();
        }
        // Erase all rows below
        for r in (row + 1)..self.rows {
            for c in 0..self.cols {
                self.cell_mut(r, c).reset();
            }
        }
    }

    /// Erase from start of display to cursor.
    pub fn erase_above(&mut self, row: usize, col: usize) {
        // Erase all rows above
        for r in 0..row {
            for c in 0..self.cols {
                self.cell_mut(r, c).reset();
            }
        }
        // Erase from start of current row to cursor
        for c in 0..=col.min(self.cols - 1) {
            self.cell_mut(row, c).reset();
        }
    }

    /// Erase entire display.
    pub fn erase_all(&mut self) {
        for cell in &mut self.cells {
            cell.reset();
        }
    }

    /// Erase scrollback buffer.
    pub fn erase_scrollback(&mut self) {
        self.scrollback.clear();
    }

    /// Erase from cursor to end of line.
    pub fn erase_line_right(&mut self, row: usize, col: usize) {
        for c in col..self.cols {
            self.cell_mut(row, c).reset();
        }
    }

    /// Erase from start of line to cursor.
    pub fn erase_line_left(&mut self, row: usize, col: usize) {
        for c in 0..=col.min(self.cols - 1) {
            self.cell_mut(row, c).reset();
        }
    }

    /// Erase entire line.
    pub fn erase_line_all(&mut self, row: usize) {
        for c in 0..self.cols {
            self.cell_mut(row, c).reset();
        }
    }

    /// Insert n blank lines at cursor row, pushing content down.
    pub fn insert_lines(&mut self, row: usize, n: usize, bottom: usize) {
        let n = n.min(bottom - row + 1);
        for _ in 0..n {
            // Shift rows down
            for r in (row + 1..=bottom).rev() {
                let src_start = (r - 1) * self.cols;
                let dst_start = r * self.cols;
                for c in 0..self.cols {
                    self.cells[dst_start + c] = self.cells[src_start + c].clone();
                }
            }
            // Clear the inserted row
            for c in 0..self.cols {
                self.cell_mut(row, c).reset();
            }
        }
    }

    /// Delete n lines at cursor row, pulling content up.
    pub fn delete_lines(&mut self, row: usize, n: usize, bottom: usize) {
        let n = n.min(bottom - row + 1);
        for _ in 0..n {
            for r in row..bottom {
                let src_start = (r + 1) * self.cols;
                let dst_start = r * self.cols;
                for c in 0..self.cols {
                    self.cells[dst_start + c] = self.cells[src_start + c].clone();
                }
            }
            for c in 0..self.cols {
                self.cell_mut(bottom, c).reset();
            }
        }
    }

    /// Insert n blank characters at cursor position.
    pub fn insert_chars(&mut self, row: usize, col: usize, n: usize) {
        let n = n.min(self.cols - col);
        let start = row * self.cols;
        // Shift characters right
        for c in (col + n..self.cols).rev() {
            self.cells[start + c] = self.cells[start + c - n].clone();
        }
        // Clear inserted positions
        for c in col..col + n {
            self.cells[start + c] = Cell::default();
        }
    }

    /// Delete n characters at cursor position.
    pub fn delete_chars(&mut self, row: usize, col: usize, n: usize) {
        let n = n.min(self.cols - col);
        let start = row * self.cols;
        // Shift characters left
        for c in col..self.cols - n {
            self.cells[start + c] = self.cells[start + c + n].clone();
        }
        // Clear vacated positions
        for c in (self.cols - n)..self.cols {
            self.cells[start + c] = Cell::default();
        }
    }

    /// Resize the grid.
    pub fn resize(&mut self, new_cols: usize, new_rows: usize) {
        let mut new_cells = vec![Cell::default(); new_cols * new_rows];

        // Copy existing content
        let copy_rows = self.rows.min(new_rows);
        let copy_cols = self.cols.min(new_cols);
        for row in 0..copy_rows {
            for col in 0..copy_cols {
                new_cells[row * new_cols + col] = self.cells[row * self.cols + col].clone();
            }
        }

        self.cells = new_cells;
        self.cols = new_cols;
        self.rows = new_rows;
        self.scroll_top = 0;
        self.scroll_bottom = new_rows - 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_new() {
        let grid = Grid::new(80, 24);
        assert_eq!(grid.cols, 80);
        assert_eq!(grid.rows, 24);
        assert_eq!(grid.cell(0, 0).content.as_str(), " ");
    }

    #[test]
    fn test_cell_set() {
        let mut grid = Grid::new(80, 24);
        grid.cell_mut(0, 0).set('A', &CellAttrs::default());
        assert_eq!(grid.cell(0, 0).content.as_str(), "A");
    }

    #[test]
    fn test_scroll_up() {
        let mut grid = Grid::new(3, 3);
        grid.cell_mut(0, 0).set('A', &CellAttrs::default());
        grid.cell_mut(1, 0).set('B', &CellAttrs::default());
        grid.cell_mut(2, 0).set('C', &CellAttrs::default());

        grid.scroll_up(1);

        assert_eq!(grid.cell(0, 0).content.as_str(), "B");
        assert_eq!(grid.cell(1, 0).content.as_str(), "C");
        assert_eq!(grid.cell(2, 0).content.as_str(), " ");
        assert_eq!(grid.scrollback.len(), 1);
    }

    #[test]
    fn test_erase_all() {
        let mut grid = Grid::new(3, 3);
        grid.cell_mut(0, 0).set('A', &CellAttrs::default());
        grid.erase_all();
        assert_eq!(grid.cell(0, 0).content.as_str(), " ");
    }

    #[test]
    fn test_insert_chars() {
        let mut grid = Grid::new(5, 1);
        grid.cell_mut(0, 0).set('A', &CellAttrs::default());
        grid.cell_mut(0, 1).set('B', &CellAttrs::default());
        grid.cell_mut(0, 2).set('C', &CellAttrs::default());

        grid.insert_chars(0, 1, 1);

        assert_eq!(grid.cell(0, 0).content.as_str(), "A");
        assert_eq!(grid.cell(0, 1).content.as_str(), " ");
        assert_eq!(grid.cell(0, 2).content.as_str(), "B");
        assert_eq!(grid.cell(0, 3).content.as_str(), "C");
    }

    #[test]
    fn test_delete_chars() {
        let mut grid = Grid::new(5, 1);
        grid.cell_mut(0, 0).set('A', &CellAttrs::default());
        grid.cell_mut(0, 1).set('B', &CellAttrs::default());
        grid.cell_mut(0, 2).set('C', &CellAttrs::default());

        grid.delete_chars(0, 1, 1);

        assert_eq!(grid.cell(0, 0).content.as_str(), "A");
        assert_eq!(grid.cell(0, 1).content.as_str(), "C");
        assert_eq!(grid.cell(0, 2).content.as_str(), " ");
    }

    #[test]
    fn test_resize() {
        let mut grid = Grid::new(3, 3);
        grid.cell_mut(0, 0).set('A', &CellAttrs::default());
        grid.resize(5, 5);
        assert_eq!(grid.cols, 5);
        assert_eq!(grid.rows, 5);
        assert_eq!(grid.cell(0, 0).content.as_str(), "A");
    }
}
