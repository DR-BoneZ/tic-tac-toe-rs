#[macro_use]
extern crate failure;

use failure::Error;
use itertools::Itertools;
use std::fmt;
use tokio::prelude::*;

use Mark::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Square(Option<Mark>);
impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Some(a) => write!(f, "{}", a),
            None => write!(f, " "),
        }
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

#[derive(Clone, Debug, PartialEq, Eq)]
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
                Winner::Tie => "Thanos wins.",
            }
        )
    }
}

#[derive(Debug)]
struct EndgameCount {
    wins: usize,
    thanos_wins: usize,
    losses: usize,
}
impl EndgameCount {
    pub fn new() -> Self {
        EndgameCount {
            wins: 0,
            thanos_wins: 0,
            losses: 0,
        }
    }
}

#[derive(Clone)]
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

    pub fn open<'a>(&'a self) -> impl Iterator<Item = (usize, usize)> + 'a {
        self.board.iter().enumerate().flat_map(|(y, i)| {
            i.iter()
                .enumerate()
                .filter(|(_, sq)| sq.is_none())
                .map(move |(x, _)| (x, y))
        })
    }

    pub fn aggregate_wins(&self, count: &mut EndgameCount, marker: Mark) {
        let coord_iter = self.open();
        if let Some(ref winner) = self.status {
            if winner == &From::from(marker) {
                count.wins += 1;
            } else if winner == &Winner::Tie {
                count.thanos_wins += 1;
            } else {
                count.losses += 1;
            }
        } else {
            for (x, y) in coord_iter {
                let mut b = self.clone();
                b.set(x, y, b.turn);
                b.advance();
                b.aggregate_wins(count, marker);
            }
        }
    }

    pub fn get_ai_move(
        &self,
    ) -> impl Future<Item = (usize, usize), Error = Error> {
        fn sort_fn(
            a: &(usize, usize, EndgameCount),
            b: &(usize, usize, EndgameCount),
        ) -> std::cmp::Ordering {
            use std::cmp::Ordering::*;
            match a.2.losses.cmp(&b.2.losses) {
                Equal => match a.2.wins.cmp(&b.2.wins) {
                    Equal => a.2.thanos_wins.cmp(&b.2.thanos_wins),
                    ne => ne.reverse(),
                },
                ne => ne,
            }
        }

        let board = self.clone();
        let coord_iter = self.open().collect::<Vec<_>>().into_iter();
        let turn = self.turn;
        future::join_all(
            coord_iter
                .map(move |(x, y)| {
                    let board = board.clone();
                    future::lazy(move || {
                        let mut b = board;
                        b.set(x, y, b.turn);
                        b.advance();
                        let mut count = EndgameCount::new();
                        b.aggregate_wins(&mut count, turn);
                        future::ok((x, y, count))
                    })
                })
                .collect::<Vec<_>>(),
        )
        .and_then(|inner| {
            future::result(
                inner
                    .into_iter()
                    .sorted_by(sort_fn)
                    .map(|a| {
                        println!("{:?}", a);
                        a
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|(x, y, _)| (x, y))
                    .next()
                    .ok_or(format_err!("can't move")),
            )
        })
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

        if (x + y) % 2 == 0 {
            if self.board[0][2] == self.board[1][1]
                && self.board[1][1] == self.board[2][0]
                && self.board[2][0] == From::from(val)
            {
                self.status = Some(From::from(val));
            }
            if self.board[0][0] == self.board[1][1]
                && self.board[1][1] == self.board[2][2]
                && self.board[2][2] == From::from(val)
            {
                self.status = Some(From::from(val));
            }

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

    let mut b = Game::new();
    let mut rt = tokio::runtime::Runtime::new()?;

    println!("{}", b);
    while !b.is_done() {
        if b.turn == O {
            let coord = rt.block_on(b.get_ai_move())?;
            b.set(coord.0, coord.1, b.turn)?;
        } else {
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

        }
        println!("{}", b);

        b.advance();
    }
    println!("{}", b.status.unwrap_or(Winner::Tie));

    Ok(())
}
