use chess::Board;
use std::ffi::CString;

pub fn init() {
    let path = CString::new("model.nnue").unwrap();
    assert!(unsafe { probe::nnue_init(path.as_ptr()) })
}

pub fn eval(board: &Board) -> i32 {
    let fen = CString::new(board.to_string()).unwrap();
    unsafe { probe::nnue_evaluate_fen(fen.as_ptr()) }
}

#[cxx::bridge]
mod probe {
    extern "C++" {
        include!("chevii/src/ai/nnue/nnue.h");

        pub unsafe fn nnue_init(path: *const c_char) -> bool;
        pub unsafe fn nnue_evaluate_fen(fen: *const c_char) -> i32;
    }
}
