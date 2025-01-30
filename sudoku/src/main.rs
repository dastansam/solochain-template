//! Create a sudoku tester and make the test pass.

use std::str::FromStr;

/// The size of the classic sudoku board
const SUDOKU_SIZE: usize = 9;

/// Represents a sudoku sequence.
type SudokuSequence = [u8; SUDOKU_SIZE];
/// A sudoku board is 9 rows of
type SudokuBoard = [SudokuSequence; SUDOKU_SIZE];

/// Sudoku rules
///
/// - Each row should have numbers 1-9, no repeats.
/// - Each column should have numbers 1-9, no repeats.
/// - Each 3x3 quadrant should have numbers 1-9, no repeats
#[derive(Debug)]
struct Sudoku {
    board: SudokuBoard,
}

impl Sudoku {
    /// Validate the sudoku state
    fn valid(&self) -> bool {
        // `u8` is not enough to hold numbers from 1..9, so we use the leading 9 bits of `u16`
        let mut row_visitor = [0u16; SUDOKU_SIZE];
        let mut column_visitor = [0u16; SUDOKU_SIZE];
        let mut quad_visitor = [0u16; SUDOKU_SIZE];

        for i in 0..SUDOKU_SIZE {
            for k in 0..SUDOKU_SIZE {
                let value = self.board[i][k];

                // we already do the validation while parsing, but still worth the check
                if value == 0 || value > SUDOKU_SIZE as u8 {
                    return false;
                }

                // offset by one, and subtration is safe due to above check
                let value_as_index = (self.board[i][k] - 1) as usize;

                // if `value_as_index` is already masked in a row visitor, we found a duplicate, early return
                if (row_visitor[i] & (1 << value_as_index)) != 0 {
                    return false;
                }
                row_visitor[i] |= 1 << value_as_index;

                if (column_visitor[k] & (1 << value_as_index)) != 0 {
                    return false;
                }
                column_visitor[k] |= 1 << value_as_index;

                // max value of `i = 8` and `k = 8`, so it's guaranteed that it won't be index out of range
                // basically, finds which quadrant cell's value belongs to
                let quad_idx = (i / 3) * 3 + k / 3;
                if (quad_visitor[quad_idx] & (1 << value_as_index)) != 0 {
                    return false;
                }

                quad_visitor[quad_idx] |= 1 << value_as_index;
            }
        }

        println!("final state {:?}", row_visitor);
        println!("final state {:?}", column_visitor);
        println!("final state {:?}", quad_visitor);

        true
    }
}

impl FromStr for Sudoku {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut board = [[0u8; 9]; 9];
        for (row, line) in s.split("\n").enumerate() {
            for (col, value) in line.chars().enumerate() {
                if value.is_ascii_digit() {
                    board[row][col] = value as u8 - b'0';
                } else {
                    return Err(());
                }
            }
        }

        Ok(Self { board })
    }
}

fn main() {
    println!("Validates sudoku. Look for tests");
}

#[cfg(test)]
mod tests {
    const VALID_BOARD: &str = "534678912\n\
                                672195348\n\
                                198342567\n\
                                859761423\n\
                                426853791\n\
                                713924856\n\
                                961537284\n\
                                287419635\n\
                                345286179";

    const INVALID_DUPLICATE: &str = "534678912\n\
                                    672195348\n\
                                    198342567\n\
                                    859761423\n\
                                    426853791\n\
                                    713924856\n\
                                    961537284\n\
                                    287419635\n\
                                    345286171";

    const INVALID_ZERO: &str = "534678912\n\
                                672195348\n\
                                198342567\n\
                                859761423\n\
                                426803791\n\
                                713924856\n\
                                961537284\n\
                                287419635\n\
                                345286179";

    const INVALID_ROW_PLACEMENT: &str = "534678912\n\
                                        672195348\n\
                                        198342567\n\
                                        859761423\n\
                                        426853791\n\
                                        713924856\n\
                                        961537284\n\
                                        287419635\n\
                                        345286971"; // Last row has 9,7 swapped but all valid numbers

    const INVALID_COLUMN_PLACEMENT: &str = "534678912\n\
                                            672195348\n\
                                            198342567\n\
                                            859761423\n\
                                            426853791\n\
                                            713924856\n\
                                            961537284\n\
                                            287416935\n\
                                            345289171"; // Last two columns swapped but valid numbers

    const INVALID_QUADRANT: &str = "534678912\n\
                                    672195348\n\
                                    198342567\n\
                                    859761423\n\
                                    426853791\n\
                                    713924856\n\
                                    961537284\n\
                                    287419635\n\
                                    342586179"; // Bottom right quadrant numbers rearranged

    use super::*;

    fn parse_and_validate(board: &str) -> bool {
        let sudoku: Sudoku = board.parse().expect("failed parsing sudoku");

        sudoku.valid()
    }

    #[test]
    fn test_validate_sudoku() {
        assert!(parse_and_validate(VALID_BOARD));
    }

    #[test]
    fn invalid_row() {
        assert!(!parse_and_validate(INVALID_ROW_PLACEMENT));
    }

    #[test]
    fn invalid_quadrant() {
        assert!(!parse_and_validate(INVALID_QUADRANT))
    }

    #[test]
    fn invalid_col() {
        assert!(!parse_and_validate(INVALID_COLUMN_PLACEMENT))
    }

    #[test]
    fn test_bitmasking() {
        let mut n = 0u16;
        n |= 1 << 2;

        println!("{:?}", n);

        println!("is masked {:?}", (n & (1 << 1)) != 0);
        println!("test {:?}", 1 << 9);
    }
}
