//! Pretty printing of matrices and vectors. Useful for debugging and visualizing the data.

// TODO: create a macro for pretty printing, that can take the name of the matrix, and
// print its name and size in the top border of the pretty print.

use super::prelude::*;

const CELL_WIDTH: usize = 14;

const RESET_TEXT: &str = "\x1b[0m";
const RED_TEXT: &str = "\x1b[31m";
const GREEN_TEXT: &str = "\x1b[32m";
const BLUE_TEXT: &str = "\x1b[34m";
const YELLOW_TEXT: &str = "\x1b[33m";
const MAGENTA_TEXT: &str = "\x1b[35m";
const CYAN_TEXT: &str = "\x1b[36m";
const BOLD_TEXT: &str = "\x1b[1m";
const UNDERLINE_TEXT: &str = "\x1b[4m";
const ITALIC_TEXT: &str = "\x1b[3m";

const BAR: char = '│';
const UPPER_LEFT_CORNER: char = '╭';
const UPPER_RIGHT_CORNER: char = '╮';
const LOWER_LEFT_CORNER: char = '╰';
const LOWER_RIGHT_CORNER: char = '╯';
// const HORIZONTAL_LINE: char = '─';

fn print_cell<T: GbpFloat>(x: T) {
    if x.is_zero() {
        print!("{}{:14.2}{}", RESET_TEXT, x, RESET_TEXT);
    } else if x.is_sign_negative() {
        print!("{}{:14.2}{}", RED_TEXT, x, RESET_TEXT);
    } else if x.is_sign_positive() {
        print!("{}{:14.2}{}", GREEN_TEXT, x, RESET_TEXT);
    } else if x.is_nan() {
        print!("{}{:14.2}{}", MAGENTA_TEXT, x, RESET_TEXT);
    } else if x.is_infinite() {
        print!("{}{:14.2}{}", YELLOW_TEXT, x, RESET_TEXT);
    }
}

pub fn _pretty_print_matrix<T, M>(
    matrix: &M,
    name: Option<&str>,
    file: Option<&str>,
    line: Option<u32>,
) where
    T: GbpFloat,
    M: PrettyPrintMatrix<T>,
{
    let (nrows, ncols) = matrix.shape();
    let cell_columns = CELL_WIDTH * ncols;
    let dims = format!("{}x{}", nrows, ncols);
    let horizontal_line = "─".repeat(cell_columns);

    // match (file, line, name) {
    //     (Some(file), Some(line), Some(name)) => {
    //         println!("{}:{}{}{}", file, YELLOW_TEXT, line, RESET_TEXT);
    //         println!("{}{}{}{}{}{}{}{}{}", UPPER_LEFT_CORNER, CYAN_TEXT, name, RESET_TEXT, "─".repeat(cell_columns - name.len() - dims.len()), MAGENTA_TEXT, dims, RESET_TEXT, UPPER_RIGHT_CORNER);

    //         },
    //         (None, None, Some(name)) => {
    //             println!("{}{}{}{}{}{}{}{}{}", UPPER_LEFT_CORNER, CYAN_TEXT, name, RESET_TEXT, "─".repeat(cell_columns - name.len() - dims.len()), MAGENTA_TEXT, dims, RESET_TEXT, UPPER_RIGHT_CORNER);

    //         },
    //         _ => {
    //             println!("{}{}{}", UPPER_LEFT_CORNER, horizontal_line, UPPER_RIGHT_CORNER);

    //         }
    //         }

    if let (Some(file), Some(line)) = (file, line) {
        println!("{}:{}{}{}", file, YELLOW_TEXT, line, RESET_TEXT);
    }

    if let Some(name) = name {
        // TODO: handle if name is longer than cell_columns
        println!(
            "{}{}{}{}{}{}{}{}{}",
            UPPER_LEFT_CORNER,
            CYAN_TEXT,
            name,
            RESET_TEXT,
            "─".repeat(cell_columns - name.len() - dims.len()),
            MAGENTA_TEXT,
            dims,
            RESET_TEXT,
            UPPER_RIGHT_CORNER
        );
    } else {
        println!(
            "{}{}{}",
            UPPER_LEFT_CORNER, horizontal_line, UPPER_RIGHT_CORNER
        );
    }

    for i in 0..nrows {
        print!("{}", BAR);
        for j in 0..ncols {
            let x = matrix.at(i, j);
            print_cell(x);
        }
        println!("{}", BAR);
    }
    println!(
        "{}{}{}",
        LOWER_LEFT_CORNER, horizontal_line, LOWER_RIGHT_CORNER
    );
}

pub fn _pretty_print_vector<T, V>(
    vector: &V,
    name: Option<&str>,
    file: Option<&str>,
    line: Option<u32>,
) where
    T: GbpFloat,
    V: PrettyPrintVector<T>,
{
    let cell_columns = CELL_WIDTH * vector.len();
    let horizontal_line = "─".repeat(cell_columns);
    let dims = format!(" {}x{} ", vector.len(), 1);

    if let (Some(file), Some(line)) = (file, line) {
        println!("{}:{}{}{}", file, YELLOW_TEXT, line, RESET_TEXT);
    }

    if let Some(name) = name {
        println!(
            "{}{}{}{}{}{}{}{}{}",
            UPPER_LEFT_CORNER,
            CYAN_TEXT,
            name,
            RESET_TEXT,
            "─".repeat(cell_columns - name.len() - dims.len()),
            MAGENTA_TEXT,
            dims,
            RESET_TEXT,
            UPPER_RIGHT_CORNER
        );
    } else {
        println!(
            "{}{}{}",
            UPPER_LEFT_CORNER, horizontal_line, UPPER_RIGHT_CORNER
        );
    }

    print!("{}", BAR);

    for i in 0..vector.len() {
        let x = vector.at(i);
        print_cell(x);
    }
    println!(
        "{}\n{}{}{}",
        BAR, LOWER_LEFT_CORNER, horizontal_line, LOWER_RIGHT_CORNER
    );
}

pub trait PrettyPrintVector<T: GbpFloat>: Sized {
    /// Returns the length of the vector.
    fn len(&self) -> usize;
    /// Returns the element at index `i`.
    fn at(&self, i: usize) -> T;

    #[inline(always)]
    fn pretty_print(&self) {
        _pretty_print_vector(self, None, None, None);
    }
}

impl<T: GbpFloat> PrettyPrintVector<T> for Vector<T> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn at(&self, i: usize) -> T {
        self[i]
    }
}
pub trait PrettyPrintMatrix<T: GbpFloat>: Sized {
    /// Returns the shape of the matrix as a tuple `(nrows, ncols)`.
    fn shape(&self) -> (usize, usize);
    /// Returns the element at index `(i, j)`.
    fn at(&self, i: usize, j: usize) -> T;

    #[inline(always)]
    fn pretty_print(&self) {
        _pretty_print_matrix(self, None, None, None);
    }
}

impl<T: GbpFloat> PrettyPrintMatrix<T> for Matrix<T> {
    #[inline(always)]
    fn shape(&self) -> (usize, usize) {
        (self.nrows(), self.ncols())
    }

    #[inline(always)]
    fn at(&self, i: usize, j: usize) -> T {
        self[(i, j)]
    }
}

#[macro_export]
macro_rules! pretty_print_vector {
    ($name:expr) => {
        // TODO: add line number and file name to the macro
        // $name.pretty_print(Some(stringify!($name)));
        $crate::pretty_print::_pretty_print_vector(
            $name,
            Some(stringify!($name)),
            Some(file!()),
            Some(line!()),
        );
        // ::gbp_linalg::pretty_print::_pretty_print_vector($name, Some(stringify!($name)));
    };
}

#[macro_export]
macro_rules! pretty_print_matrix {
    ($name:expr) => {
        $crate::pretty_print::_pretty_print_matrix(
            $name,
            Some(stringify!($name)),
            Some(file!()),
            Some(line!()),
        );
    };
}
