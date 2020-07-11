
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
    curve: Option<String>,
}

fn main() {
    let args = RogFanCurveCli::from_args();
    
    let mut curve = Curve::new();
    
    curve.set_point(0, 0x1e, 0x00);
    curve.set_point(1, 0x2d, 0x01);
    curve.set_point(2, 0x32, 0x04);
    curve.set_point(3, 0x3c, 0x04);
    curve.set_point(4, 0x46, 0x13);
    curve.set_point(5, 0x50, 0x40);
    curve.set_point(6, 0x5a, 0x64);
    curve.set_point(7, 0x64, 0x64);
    
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
    
    curve.apply(board, Fan::Cpu).unwrap();
    curve.apply(board, Fan::Gpu).unwrap();
}
