//! An attempt to create a n-body simulation that might become a game one day.
//! Inspired by the book "The Three Body Problem" by Liu Cixin.

mod game;
use game::start;

mod galaxy;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "nbody", about = "N bodies in space")]

struct Opt {
    #[structopt(short, long, default_value = "3")]
    suns: u32,
}

fn main() -> ggez::GameResult {
    let opt = Opt::from_args();
    start(opt.suns)
}
