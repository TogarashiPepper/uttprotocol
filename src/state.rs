//! Module for parsing UTT (Ultimate Tic Tac Toe) strings
//! UTT strings consist of:
//! - a single digit denoting the active board index (0-8, with 9 meaning "any board") and a slash
//! - 9 series of 9 X's, O's, or _'s, separated by slashes
//! some additional features:
//! - if there is a run of multiple of the same character (e.g. XXXX or OOOOOOO) it may be replaced by
//! the length of the run followed by that character (e.g. 4X or 7O), runs must be 1..=9.
//! - the last slash may be optionally succeeded by a move of the form [a..=i][1..=9] (e.g. a1 or g9)
//! to denote the most recent move played

use chumsky::prelude::*;
use std::ops::RangeInclusive;

#[derive(Debug, Clone, Copy)]
pub enum Square {
	Empty,
	X,
	O,
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
	row: u8,
	column: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum MoveErr {
	InvalidRow,
	InvalidColumn,
}

impl Move {
	pub fn new(row: u8, col: u8) -> Result<Self, MoveErr> {
		if !(0..=8).contains(&row) {
			Err(MoveErr::InvalidRow)
		} else if !(0..=8).contains(&col) {
			Err(MoveErr::InvalidColumn)
		} else {
			Ok(Self { row, column: col })
		}
	}

	pub fn row(self) -> u8 {
		self.row
	}

	pub fn col(self) -> u8 {
		self.column
	}
}

#[derive(Debug, Clone)]
pub struct State {
	pub active: u8,
	pub squares: [Square; 81],
	pub last_move: Option<Move>,
}

fn _parse<'a>() -> impl Parser<'a, &'a str, State, extra::Err<Rich<'a, char>>> {
	let digit = one_of('0'..='9').map(|c: char| c.to_digit(10).unwrap() as usize);
	let slash = just('/');

	let cell = choice((
		just('X').to(Square::X),
		just('O').to(Square::O),
		just('_').to(Square::Empty),
	));

	let run = choice((
		cell.map(|c| vec![c]),
		digit
			.clone()
			.then(cell)
			.map(|(run_len, cell)| vec![cell; run_len]),
	));

	let row = run
		.repeated()
		.at_least(1)
		.collect()
		.try_map(|runs: Vec<Vec<Square>>, span| {
			let cells: Vec<_> = runs.into_iter().flatten().collect();

			if cells.len() != 9 {
				Err(Rich::custom(
					span,
					format!("Board must have exactly 9 squares, got: {}", cells.len()),
				))
			} else {
				Ok(cells)
			}
		});

	let active_brd = digit.clone().then_ignore(slash);
	let boards = row
		.separated_by(slash)
		.exactly(9)
		.collect::<Vec<Vec<Square>>>();

	let last_move = slash
		.or_not()
		.ignore_then(one_of('a'..='i').map(|c: char| c as u32 - b'a' as u32))
		.then(digit.clone())
		.map(|(board, index)| Move::new(board as u8, index as u8).unwrap())
		.or_not();

	active_brd
		.then(boards)
		.then(last_move)
		.then_ignore(end())
		.map(|((active, boards), last_move)| State {
			active: active as u8,
			squares: boards
				.into_iter()
				.flatten()
				.collect::<Vec<Square>>()
				.try_into()
				.unwrap(),
			last_move,
		})
}

// Wrapper around chumsky parser so we can change it later in a non-breaking way
fn parse(input: &str) -> Result<State, Vec<(RangeInclusive<usize>, String)>> {
	let res = _parse().parse(input);
	if res.has_errors() {
		let errs = res
			.into_errors()
			.into_iter()
			.map(|e| {
				let sp = e.span();
				let span = sp.start..=sp.end;
				let reason = e.into_reason().to_string();

				(span, reason)
			})
			.collect();

		Err(errs)
	} else {
		Ok(res.into_output().unwrap())
	}
}
