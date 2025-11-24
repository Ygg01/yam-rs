use alloc::collections::VecDeque;



pub struct Scanner<'input, T> {
    input: T,
    cursor_pos: usize,
    tokens: VecDeque<Token<'input>>
}