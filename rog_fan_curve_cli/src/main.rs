
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
    curve: Option<String>,
}

fn main() {
    let args = RogFanCurveCli::from_args();
    
    let mut curve = Curve::new();
    
    curve.set_point(0,  30,   0);
    curve.set_point(1,  40,   1);
    curve.set_point(2,  50,   4);
    curve.set_point(3,  60,   4);
    curve.set_point(4,  70,  13);
    curve.set_point(5,  80,  40);
    curve.set_point(6,  90, 100);
    curve.set_point(7, 100, 100);
    
    if let Some(curve_str) = args.curve {
        for (i, point_str) in curve_str.split(",").enumerate() {
            let mut parts = point_str.split(":");
            let temp: u8 = parts.next().unwrap().parse().unwrap();
            let speed: u8 = parts.next().unwrap().parse().unwrap();
            curve.set_point(i as u8, temp, speed);
        }
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
    
    if cpu {
        curve.apply(board, Fan::Cpu).unwrap();
    }
    if gpu {
        curve.apply(board, Fan::Gpu).unwrap();
    }
}
