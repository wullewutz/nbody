//! An attempt to create a n-body simulation that might become a game one day.
//! Inspired by the book "The Three Body Problem" by Liu Cixin.

mod game;
use game::start;

mod galaxy;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "nbody")]
/// N bodies in a dark forest called space...
///
/// Key bindings:
///
/// w/s/a/d - move up/down/left/right
///
/// Space - pause/resume
///
/// +/- - faster/slower
///
/// i/o - zoom in/out.
///
/// q - quit
struct Opt {
    #[structopt(short, long, default_value = "3")]
    suns: u32,
}

fn main() -> ggez::GameResult {
    let opt = Opt::from_args();
    start(opt.suns)
}
