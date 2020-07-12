
use structopt::StructOpt;

use rog_fan_curve::{
    Curve,
    Board,
    Fan,
};

#[derive(StructOpt)]
struct RogFanCurveCli {
    #[structopt(long = "board")]
    board: Option<String>,
    #[structopt(long)]
    cpu: bool,
    #[structopt(long)]
    gpu: bool,
    #[structopt(long)]
    no_warn: bool,
    curve: Option<String>,
}

fn main() {
    let args = RogFanCurveCli::from_args();
    
    let mut curve = Curve::new();
    
    curve.set_point(0,  30,   0);
    curve.set_point(1,  40,   1);
    curve.set_point(2,  50,   4);
    curve.set_point(3,  60,   4);
    curve.set_point(4,  70,  40);
    curve.set_point(5,  80,  60);
    curve.set_point(6,  90, 100);
    curve.set_point(7, 100, 100);
    
    if let Some(config_str) = args.curve {
        curve = Curve::from_config_str(&config_str).unwrap();
    }
    
    let mut board = args.board.as_ref().map(|name| Board::from_name(name).expect("unknown board"));
    
    if let None = board {
        board = Board::from_board_name();
    }
    
    let board = board.expect("unknown board");
    
    let mut cpu = args.cpu;
    let mut gpu = args.gpu;
    if !cpu && !gpu {
        cpu = true;
        gpu = true;
    }
    
    let mut warning_given = false;
    if args.no_warn {
        warning_given = true;
    }
    
    if cpu {
        if !warning_given && curve.check_safety(Fan::Cpu).is_err() {
            warning_given = true;
            eprintln!("Warning: This fan curve wouldn't be allowed in armoury crate and may be unsafe.");
        }
        curve.apply(board, Fan::Cpu).unwrap();
    }
    if gpu {
        if !warning_given && curve.check_safety(Fan::Gpu).is_err() {
            #[allow(unused_assignments)]
            { warning_given = true; }
            eprintln!("Warning: This fan curve wouldn't be allowed in armoury crate and may be unsafe.");
        }
        curve.apply(board, Fan::Gpu).unwrap();
    }
}
