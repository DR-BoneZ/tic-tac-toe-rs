#[macro_use]
extern crate failure;

use failure::Error;
use itertools::Itertools;
use std::fmt;

use Mark::*;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    X,
    O,
}
impl fmt::Display for Mark {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                X => "X",
                O => "O",
            }
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Square(Option<Mark>);
impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.0 {
                Some(a) => format!("{}", a),
                None => " ".to_owned(),
            }
        )
    }
}
impl std::ops::Deref for Square {
    type Target = Option<Mark>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<Mark> for Square {
    fn from(m: Mark) -> Self {
        Square(Some(m))
    }
}

#[derive(Debug)]
enum Winner {
    XWins,
    OWins,
    Tie,
}
impl From<Mark> for Winner {
    fn from(m: Mark) -> Self {
        match m {
            X => Winner::XWins,
            O => Winner::OWins,
        }
    }
}
impl fmt::Display for Winner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Winner::XWins => "X Wins!",
                Winner::OWins => "O Wins!",
                Winner::Tie => "No one wins, ya losers.",
            }
        )
    }
}

struct Game {
    board: [[Square; 3]; 3],
    status: Option<Winner>,
    pub turn: Mark,
}
impl Game {
    pub fn new() -> Self {
        Game {
            board: [[Square(None); 3]; 3],
            status: None,
            turn: X,
        }
    }

    pub fn advance(&mut self) -> Mark {
        self.turn = match self.turn {
            X => O,
            O => X,
        };
        self.turn
    }

    pub fn set(
        &mut self,
        x: usize,
        y: usize,
        val: Mark,
    ) -> Result<&mut Self, Error> {
        ensure!(x < 3, "x out of bounds");
        ensure!(y < 3, "y out of bounds");
        if !self.board[y][x].is_none() {
            bail!("Cannot mark a non-empty space");
        }
        self.board[y][x] = From::from(val);
        if self.board[y]
            .iter()
            .fold(true, |acc, sq| acc && sq == &Square::from(val))
        {
            self.status = Some(From::from(val))
        }
        if self
            .board
            .iter()
            .map(|r| r[x])
            .fold(true, |acc, sq| acc && sq == Square::from(val))
        {
            self.status = Some(From::from(val))
        }
        match x + y {
            2 => {
                if self.board[0][2] == self.board[1][1]
                    && self.board[1][1] == self.board[2][0]
                {
                    self.status = Some(From::from(val));
                }
            }
            0 | 4 => {
                if self.board[0][0] == self.board[1][1]
                    && self.board[1][1] == self.board[2][2]
                {
                    self.status = Some(From::from(val));
                }
            }
            _ => (),
        }
        if self.status.is_none() && self.is_full() {
            self.status = Some(Winner::Tie);
        }

        Ok(self)
    }

    pub fn is_full(&self) -> bool {
        self.board
            .iter()
            .flatten()
            .fold(true, |acc, sq| acc && sq.is_some())
    }

    pub fn is_done(&self) -> bool {
        match self.status {
            None => false,
            _ => true,
        }
    }
}
impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.board
                .iter()
                .map(|i| i
                    .iter()
                    .map(|j| format!("{}", j))
                    .intersperse("|".to_owned())
                    .collect())
                .intersperse("\n-----\n".to_owned())
                .collect::<String>()
        )
    }
}

macro_rules! unwrap_or_cont {
    ($x:expr) => {
        match $x {
            Ok(a) => a,
            Err(_) => {
                println!("I didn't understand that.");
                continue;
            }
        }
    };
}

fn main() -> Result<(), Error> {
    use std::io::stdin;
    use std::io::Read;

    let mut b = Game::new();
    while !b.is_done() {
        println!("{}: What is your move?", b.turn);
        let mut cmd = String::new();
        stdin().read_line(&mut cmd)?;
        let cmd = cmd.split(",").map(|a| a.trim()).collect::<Vec<_>>();
        match (cmd.get(0), cmd.get(1)) {
            (Some(x), Some(y)) => {
                match b.set(
                    unwrap_or_cont!(x.parse()),
                    unwrap_or_cont!(y.parse()),
                    b.turn,
                ) {
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                    _ => (),
                };
            }
            _ => {
                println!("I didn't understand that.");
                continue;
            }
        }
        println!("{}", b);

        b.advance();
    }
    println!("{}", b.status.unwrap_or(Winner::Tie));

    Ok(())
}
