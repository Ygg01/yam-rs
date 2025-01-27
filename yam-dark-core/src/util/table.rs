pub const U8_INDENT_TABLE: [[u8; 8]; 256] = [
    [0, 1, 2, 3, 4, 5, 6, 7],
    [0, 0, 1, 2, 3, 4, 5, 6],
    [0, 1, 0, 1, 2, 3, 4, 5],
    [0, 0, 0, 1, 2, 3, 4, 5],
    [0, 1, 2, 0, 1, 2, 3, 4],
    [0, 0, 1, 0, 1, 2, 3, 4],
    [0, 1, 0, 0, 1, 2, 3, 4],
    [0, 0, 0, 0, 1, 2, 3, 4],
    [0, 1, 2, 3, 0, 1, 2, 3],
    [0, 0, 1, 2, 0, 1, 2, 3],
    [0, 1, 0, 1, 0, 1, 2, 3],
    [0, 0, 0, 1, 0, 1, 2, 3],
    [0, 1, 2, 0, 0, 1, 2, 3],
    [0, 0, 1, 0, 0, 1, 2, 3],
    [0, 1, 0, 0, 0, 1, 2, 3],
    [0, 0, 0, 0, 0, 1, 2, 3],
    [0, 1, 2, 3, 4, 0, 1, 2],
    [0, 0, 1, 2, 3, 0, 1, 2],
    [0, 1, 0, 1, 2, 0, 1, 2],
    [0, 0, 0, 1, 2, 0, 1, 2],
    [0, 1, 2, 0, 1, 0, 1, 2],
    [0, 0, 1, 0, 1, 0, 1, 2],
    [0, 1, 0, 0, 1, 0, 1, 2],
    [0, 0, 0, 0, 1, 0, 1, 2],
    [0, 1, 2, 3, 0, 0, 1, 2],
    [0, 0, 1, 2, 0, 0, 1, 2],
    [0, 1, 0, 1, 0, 0, 1, 2],
    [0, 0, 0, 1, 0, 0, 1, 2],
    [0, 1, 2, 0, 0, 0, 1, 2],
    [0, 0, 1, 0, 0, 0, 1, 2],
    [0, 1, 0, 0, 0, 0, 1, 2],
    [0, 0, 0, 0, 0, 0, 1, 2],
    [0, 1, 2, 3, 4, 5, 0, 1],
    [0, 0, 1, 2, 3, 4, 0, 1],
    [0, 1, 0, 1, 2, 3, 0, 1],
    [0, 0, 0, 1, 2, 3, 0, 1],
    [0, 1, 2, 0, 1, 2, 0, 1],
    [0, 0, 1, 0, 1, 2, 0, 1],
    [0, 1, 0, 0, 1, 2, 0, 1],
    [0, 0, 0, 0, 1, 2, 0, 1],
    [0, 1, 2, 3, 0, 1, 0, 1],
    [0, 0, 1, 2, 0, 1, 0, 1],
    [0, 1, 0, 1, 0, 1, 0, 1],
    [0, 0, 0, 1, 0, 1, 0, 1],
    [0, 1, 2, 0, 0, 1, 0, 1],
    [0, 0, 1, 0, 0, 1, 0, 1],
    [0, 1, 0, 0, 0, 1, 0, 1],
    [0, 0, 0, 0, 0, 1, 0, 1],
    [0, 1, 2, 3, 4, 0, 0, 1],
    [0, 0, 1, 2, 3, 0, 0, 1],
    [0, 1, 0, 1, 2, 0, 0, 1],
    [0, 0, 0, 1, 2, 0, 0, 1],
    [0, 1, 2, 0, 1, 0, 0, 1],
    [0, 0, 1, 0, 1, 0, 0, 1],
    [0, 1, 0, 0, 1, 0, 0, 1],
    [0, 0, 0, 0, 1, 0, 0, 1],
    [0, 1, 2, 3, 0, 0, 0, 1],
    [0, 0, 1, 2, 0, 0, 0, 1],
    [0, 1, 0, 1, 0, 0, 0, 1],
    [0, 0, 0, 1, 0, 0, 0, 1],
    [0, 1, 2, 0, 0, 0, 0, 1],
    [0, 0, 1, 0, 0, 0, 0, 1],
    [0, 1, 0, 0, 0, 0, 0, 1],
    [0, 0, 0, 0, 0, 0, 0, 1],
    [0, 1, 2, 3, 4, 5, 6, 0],
    [0, 0, 1, 2, 3, 4, 5, 0],
    [0, 1, 0, 1, 2, 3, 4, 0],
    [0, 0, 0, 1, 2, 3, 4, 0],
    [0, 1, 2, 0, 1, 2, 3, 0],
    [0, 0, 1, 0, 1, 2, 3, 0],
    [0, 1, 0, 0, 1, 2, 3, 0],
    [0, 0, 0, 0, 1, 2, 3, 0],
    [0, 1, 2, 3, 0, 1, 2, 0],
    [0, 0, 1, 2, 0, 1, 2, 0],
    [0, 1, 0, 1, 0, 1, 2, 0],
    [0, 0, 0, 1, 0, 1, 2, 0],
    [0, 1, 2, 0, 0, 1, 2, 0],
    [0, 0, 1, 0, 0, 1, 2, 0],
    [0, 1, 0, 0, 0, 1, 2, 0],
    [0, 0, 0, 0, 0, 1, 2, 0],
    [0, 1, 2, 3, 4, 0, 1, 0],
    [0, 0, 1, 2, 3, 0, 1, 0],
    [0, 1, 0, 1, 2, 0, 1, 0],
    [0, 0, 0, 1, 2, 0, 1, 0],
    [0, 1, 2, 0, 1, 0, 1, 0],
    [0, 0, 1, 0, 1, 0, 1, 0],
    [0, 1, 0, 0, 1, 0, 1, 0],
    [0, 0, 0, 0, 1, 0, 1, 0],
    [0, 1, 2, 3, 0, 0, 1, 0],
    [0, 0, 1, 2, 0, 0, 1, 0],
    [0, 1, 0, 1, 0, 0, 1, 0],
    [0, 0, 0, 1, 0, 0, 1, 0],
    [0, 1, 2, 0, 0, 0, 1, 0],
    [0, 0, 1, 0, 0, 0, 1, 0],
    [0, 1, 0, 0, 0, 0, 1, 0],
    [0, 0, 0, 0, 0, 0, 1, 0],
    [0, 1, 2, 3, 4, 5, 0, 0],
    [0, 0, 1, 2, 3, 4, 0, 0],
    [0, 1, 0, 1, 2, 3, 0, 0],
    [0, 0, 0, 1, 2, 3, 0, 0],
    [0, 1, 2, 0, 1, 2, 0, 0],
    [0, 0, 1, 0, 1, 2, 0, 0],
    [0, 1, 0, 0, 1, 2, 0, 0],
    [0, 0, 0, 0, 1, 2, 0, 0],
    [0, 1, 2, 3, 0, 1, 0, 0],
    [0, 0, 1, 2, 0, 1, 0, 0],
    [0, 1, 0, 1, 0, 1, 0, 0],
    [0, 0, 0, 1, 0, 1, 0, 0],
    [0, 1, 2, 0, 0, 1, 0, 0],
    [0, 0, 1, 0, 0, 1, 0, 0],
    [0, 1, 0, 0, 0, 1, 0, 0],
    [0, 0, 0, 0, 0, 1, 0, 0],
    [0, 1, 2, 3, 4, 0, 0, 0],
    [0, 0, 1, 2, 3, 0, 0, 0],
    [0, 1, 0, 1, 2, 0, 0, 0],
    [0, 0, 0, 1, 2, 0, 0, 0],
    [0, 1, 2, 0, 1, 0, 0, 0],
    [0, 0, 1, 0, 1, 0, 0, 0],
    [0, 1, 0, 0, 1, 0, 0, 0],
    [0, 0, 0, 0, 1, 0, 0, 0],
    [0, 1, 2, 3, 0, 0, 0, 0],
    [0, 0, 1, 2, 0, 0, 0, 0],
    [0, 1, 0, 1, 0, 0, 0, 0],
    [0, 0, 0, 1, 0, 0, 0, 0],
    [0, 1, 2, 0, 0, 0, 0, 0],
    [0, 0, 1, 0, 0, 0, 0, 0],
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 2, 3, 4, 5, 6, 7],
    [0, 0, 1, 2, 3, 4, 5, 6],
    [0, 1, 0, 1, 2, 3, 4, 5],
    [0, 0, 0, 1, 2, 3, 4, 5],
    [0, 1, 2, 0, 1, 2, 3, 4],
    [0, 0, 1, 0, 1, 2, 3, 4],
    [0, 1, 0, 0, 1, 2, 3, 4],
    [0, 0, 0, 0, 1, 2, 3, 4],
    [0, 1, 2, 3, 0, 1, 2, 3],
    [0, 0, 1, 2, 0, 1, 2, 3],
    [0, 1, 0, 1, 0, 1, 2, 3],
    [0, 0, 0, 1, 0, 1, 2, 3],
    [0, 1, 2, 0, 0, 1, 2, 3],
    [0, 0, 1, 0, 0, 1, 2, 3],
    [0, 1, 0, 0, 0, 1, 2, 3],
    [0, 0, 0, 0, 0, 1, 2, 3],
    [0, 1, 2, 3, 4, 0, 1, 2],
    [0, 0, 1, 2, 3, 0, 1, 2],
    [0, 1, 0, 1, 2, 0, 1, 2],
    [0, 0, 0, 1, 2, 0, 1, 2],
    [0, 1, 2, 0, 1, 0, 1, 2],
    [0, 0, 1, 0, 1, 0, 1, 2],
    [0, 1, 0, 0, 1, 0, 1, 2],
    [0, 0, 0, 0, 1, 0, 1, 2],
    [0, 1, 2, 3, 0, 0, 1, 2],
    [0, 0, 1, 2, 0, 0, 1, 2],
    [0, 1, 0, 1, 0, 0, 1, 2],
    [0, 0, 0, 1, 0, 0, 1, 2],
    [0, 1, 2, 0, 0, 0, 1, 2],
    [0, 0, 1, 0, 0, 0, 1, 2],
    [0, 1, 0, 0, 0, 0, 1, 2],
    [0, 0, 0, 0, 0, 0, 1, 2],
    [0, 1, 2, 3, 4, 5, 0, 1],
    [0, 0, 1, 2, 3, 4, 0, 1],
    [0, 1, 0, 1, 2, 3, 0, 1],
    [0, 0, 0, 1, 2, 3, 0, 1],
    [0, 1, 2, 0, 1, 2, 0, 1],
    [0, 0, 1, 0, 1, 2, 0, 1],
    [0, 1, 0, 0, 1, 2, 0, 1],
    [0, 0, 0, 0, 1, 2, 0, 1],
    [0, 1, 2, 3, 0, 1, 0, 1],
    [0, 0, 1, 2, 0, 1, 0, 1],
    [0, 1, 0, 1, 0, 1, 0, 1],
    [0, 0, 0, 1, 0, 1, 0, 1],
    [0, 1, 2, 0, 0, 1, 0, 1],
    [0, 0, 1, 0, 0, 1, 0, 1],
    [0, 1, 0, 0, 0, 1, 0, 1],
    [0, 0, 0, 0, 0, 1, 0, 1],
    [0, 1, 2, 3, 4, 0, 0, 1],
    [0, 0, 1, 2, 3, 0, 0, 1],
    [0, 1, 0, 1, 2, 0, 0, 1],
    [0, 0, 0, 1, 2, 0, 0, 1],
    [0, 1, 2, 0, 1, 0, 0, 1],
    [0, 0, 1, 0, 1, 0, 0, 1],
    [0, 1, 0, 0, 1, 0, 0, 1],
    [0, 0, 0, 0, 1, 0, 0, 1],
    [0, 1, 2, 3, 0, 0, 0, 1],
    [0, 0, 1, 2, 0, 0, 0, 1],
    [0, 1, 0, 1, 0, 0, 0, 1],
    [0, 0, 0, 1, 0, 0, 0, 1],
    [0, 1, 2, 0, 0, 0, 0, 1],
    [0, 0, 1, 0, 0, 0, 0, 1],
    [0, 1, 0, 0, 0, 0, 0, 1],
    [0, 0, 0, 0, 0, 0, 0, 1],
    [0, 1, 2, 3, 4, 5, 6, 0],
    [0, 0, 1, 2, 3, 4, 5, 0],
    [0, 1, 0, 1, 2, 3, 4, 0],
    [0, 0, 0, 1, 2, 3, 4, 0],
    [0, 1, 2, 0, 1, 2, 3, 0],
    [0, 0, 1, 0, 1, 2, 3, 0],
    [0, 1, 0, 0, 1, 2, 3, 0],
    [0, 0, 0, 0, 1, 2, 3, 0],
    [0, 1, 2, 3, 0, 1, 2, 0],
    [0, 0, 1, 2, 0, 1, 2, 0],
    [0, 1, 0, 1, 0, 1, 2, 0],
    [0, 0, 0, 1, 0, 1, 2, 0],
    [0, 1, 2, 0, 0, 1, 2, 0],
    [0, 0, 1, 0, 0, 1, 2, 0],
    [0, 1, 0, 0, 0, 1, 2, 0],
    [0, 0, 0, 0, 0, 1, 2, 0],
    [0, 1, 2, 3, 4, 0, 1, 0],
    [0, 0, 1, 2, 3, 0, 1, 0],
    [0, 1, 0, 1, 2, 0, 1, 0],
    [0, 0, 0, 1, 2, 0, 1, 0],
    [0, 1, 2, 0, 1, 0, 1, 0],
    [0, 0, 1, 0, 1, 0, 1, 0],
    [0, 1, 0, 0, 1, 0, 1, 0],
    [0, 0, 0, 0, 1, 0, 1, 0],
    [0, 1, 2, 3, 0, 0, 1, 0],
    [0, 0, 1, 2, 0, 0, 1, 0],
    [0, 1, 0, 1, 0, 0, 1, 0],
    [0, 0, 0, 1, 0, 0, 1, 0],
    [0, 1, 2, 0, 0, 0, 1, 0],
    [0, 0, 1, 0, 0, 0, 1, 0],
    [0, 1, 0, 0, 0, 0, 1, 0],
    [0, 0, 0, 0, 0, 0, 1, 0],
    [0, 1, 2, 3, 4, 5, 0, 0],
    [0, 0, 1, 2, 3, 4, 0, 0],
    [0, 1, 0, 1, 2, 3, 0, 0],
    [0, 0, 0, 1, 2, 3, 0, 0],
    [0, 1, 2, 0, 1, 2, 0, 0],
    [0, 0, 1, 0, 1, 2, 0, 0],
    [0, 1, 0, 0, 1, 2, 0, 0],
    [0, 0, 0, 0, 1, 2, 0, 0],
    [0, 1, 2, 3, 0, 1, 0, 0],
    [0, 0, 1, 2, 0, 1, 0, 0],
    [0, 1, 0, 1, 0, 1, 0, 0],
    [0, 0, 0, 1, 0, 1, 0, 0],
    [0, 1, 2, 0, 0, 1, 0, 0],
    [0, 0, 1, 0, 0, 1, 0, 0],
    [0, 1, 0, 0, 0, 1, 0, 0],
    [0, 0, 0, 0, 0, 1, 0, 0],
    [0, 1, 2, 3, 4, 0, 0, 0],
    [0, 0, 1, 2, 3, 0, 0, 0],
    [0, 1, 0, 1, 2, 0, 0, 0],
    [0, 0, 0, 1, 2, 0, 0, 0],
    [0, 1, 2, 0, 1, 0, 0, 0],
    [0, 0, 1, 0, 1, 0, 0, 0],
    [0, 1, 0, 0, 1, 0, 0, 0],
    [0, 0, 0, 0, 1, 0, 0, 0],
    [0, 1, 2, 3, 0, 0, 0, 0],
    [0, 0, 1, 2, 0, 0, 0, 0],
    [0, 1, 0, 1, 0, 0, 0, 0],
    [0, 0, 0, 1, 0, 0, 0, 0],
    [0, 1, 2, 0, 0, 0, 0, 0],
    [0, 0, 1, 0, 0, 0, 0, 0],
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
];

pub const U8_COL_TABLE: [[u8; 8]; 256] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 1, 1, 1, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 1],
    [1, 2, 2, 2, 2, 2, 2, 2],
    [0, 0, 1, 1, 1, 1, 1, 1],
    [1, 1, 2, 2, 2, 2, 2, 2],
    [0, 1, 2, 2, 2, 2, 2, 2],
    [1, 2, 3, 3, 3, 3, 3, 3],
    [0, 0, 0, 1, 1, 1, 1, 1],
    [1, 1, 1, 2, 2, 2, 2, 2],
    [0, 1, 1, 2, 2, 2, 2, 2],
    [1, 2, 2, 3, 3, 3, 3, 3],
    [0, 0, 1, 2, 2, 2, 2, 2],
    [1, 1, 2, 3, 3, 3, 3, 3],
    [0, 1, 2, 3, 3, 3, 3, 3],
    [1, 2, 3, 4, 4, 4, 4, 4],
    [0, 0, 0, 0, 1, 1, 1, 1],
    [1, 1, 1, 1, 2, 2, 2, 2],
    [0, 1, 1, 1, 2, 2, 2, 2],
    [1, 2, 2, 2, 3, 3, 3, 3],
    [0, 0, 1, 1, 2, 2, 2, 2],
    [1, 1, 2, 2, 3, 3, 3, 3],
    [0, 1, 2, 2, 3, 3, 3, 3],
    [1, 2, 3, 3, 4, 4, 4, 4],
    [0, 0, 0, 1, 2, 2, 2, 2],
    [1, 1, 1, 2, 3, 3, 3, 3],
    [0, 1, 1, 2, 3, 3, 3, 3],
    [1, 2, 2, 3, 4, 4, 4, 4],
    [0, 0, 1, 2, 3, 3, 3, 3],
    [1, 1, 2, 3, 4, 4, 4, 4],
    [0, 1, 2, 3, 4, 4, 4, 4],
    [1, 2, 3, 4, 5, 5, 5, 5],
    [0, 0, 0, 0, 0, 1, 1, 1],
    [1, 1, 1, 1, 1, 2, 2, 2],
    [0, 1, 1, 1, 1, 2, 2, 2],
    [1, 2, 2, 2, 2, 3, 3, 3],
    [0, 0, 1, 1, 1, 2, 2, 2],
    [1, 1, 2, 2, 2, 3, 3, 3],
    [0, 1, 2, 2, 2, 3, 3, 3],
    [1, 2, 3, 3, 3, 4, 4, 4],
    [0, 0, 0, 1, 1, 2, 2, 2],
    [1, 1, 1, 2, 2, 3, 3, 3],
    [0, 1, 1, 2, 2, 3, 3, 3],
    [1, 2, 2, 3, 3, 4, 4, 4],
    [0, 0, 1, 2, 2, 3, 3, 3],
    [1, 1, 2, 3, 3, 4, 4, 4],
    [0, 1, 2, 3, 3, 4, 4, 4],
    [1, 2, 3, 4, 4, 5, 5, 5],
    [0, 0, 0, 0, 1, 2, 2, 2],
    [1, 1, 1, 1, 2, 3, 3, 3],
    [0, 1, 1, 1, 2, 3, 3, 3],
    [1, 2, 2, 2, 3, 4, 4, 4],
    [0, 0, 1, 1, 2, 3, 3, 3],
    [1, 1, 2, 2, 3, 4, 4, 4],
    [0, 1, 2, 2, 3, 4, 4, 4],
    [1, 2, 3, 3, 4, 5, 5, 5],
    [0, 0, 0, 1, 2, 3, 3, 3],
    [1, 1, 1, 2, 3, 4, 4, 4],
    [0, 1, 1, 2, 3, 4, 4, 4],
    [1, 2, 2, 3, 4, 5, 5, 5],
    [0, 0, 1, 2, 3, 4, 4, 4],
    [1, 1, 2, 3, 4, 5, 5, 5],
    [0, 1, 2, 3, 4, 5, 5, 5],
    [1, 2, 3, 4, 5, 6, 6, 6],
    [0, 0, 0, 0, 0, 0, 1, 1],
    [1, 1, 1, 1, 1, 1, 2, 2],
    [0, 1, 1, 1, 1, 1, 2, 2],
    [1, 2, 2, 2, 2, 2, 3, 3],
    [0, 0, 1, 1, 1, 1, 2, 2],
    [1, 1, 2, 2, 2, 2, 3, 3],
    [0, 1, 2, 2, 2, 2, 3, 3],
    [1, 2, 3, 3, 3, 3, 4, 4],
    [0, 0, 0, 1, 1, 1, 2, 2],
    [1, 1, 1, 2, 2, 2, 3, 3],
    [0, 1, 1, 2, 2, 2, 3, 3],
    [1, 2, 2, 3, 3, 3, 4, 4],
    [0, 0, 1, 2, 2, 2, 3, 3],
    [1, 1, 2, 3, 3, 3, 4, 4],
    [0, 1, 2, 3, 3, 3, 4, 4],
    [1, 2, 3, 4, 4, 4, 5, 5],
    [0, 0, 0, 0, 1, 1, 2, 2],
    [1, 1, 1, 1, 2, 2, 3, 3],
    [0, 1, 1, 1, 2, 2, 3, 3],
    [1, 2, 2, 2, 3, 3, 4, 4],
    [0, 0, 1, 1, 2, 2, 3, 3],
    [1, 1, 2, 2, 3, 3, 4, 4],
    [0, 1, 2, 2, 3, 3, 4, 4],
    [1, 2, 3, 3, 4, 4, 5, 5],
    [0, 0, 0, 1, 2, 2, 3, 3],
    [1, 1, 1, 2, 3, 3, 4, 4],
    [0, 1, 1, 2, 3, 3, 4, 4],
    [1, 2, 2, 3, 4, 4, 5, 5],
    [0, 0, 1, 2, 3, 3, 4, 4],
    [1, 1, 2, 3, 4, 4, 5, 5],
    [0, 1, 2, 3, 4, 4, 5, 5],
    [1, 2, 3, 4, 5, 5, 6, 6],
    [0, 0, 0, 0, 0, 1, 2, 2],
    [1, 1, 1, 1, 1, 2, 3, 3],
    [0, 1, 1, 1, 1, 2, 3, 3],
    [1, 2, 2, 2, 2, 3, 4, 4],
    [0, 0, 1, 1, 1, 2, 3, 3],
    [1, 1, 2, 2, 2, 3, 4, 4],
    [0, 1, 2, 2, 2, 3, 4, 4],
    [1, 2, 3, 3, 3, 4, 5, 5],
    [0, 0, 0, 1, 1, 2, 3, 3],
    [1, 1, 1, 2, 2, 3, 4, 4],
    [0, 1, 1, 2, 2, 3, 4, 4],
    [1, 2, 2, 3, 3, 4, 5, 5],
    [0, 0, 1, 2, 2, 3, 4, 4],
    [1, 1, 2, 3, 3, 4, 5, 5],
    [0, 1, 2, 3, 3, 4, 5, 5],
    [1, 2, 3, 4, 4, 5, 6, 6],
    [0, 0, 0, 0, 1, 2, 3, 3],
    [1, 1, 1, 1, 2, 3, 4, 4],
    [0, 1, 1, 1, 2, 3, 4, 4],
    [1, 2, 2, 2, 3, 4, 5, 5],
    [0, 0, 1, 1, 2, 3, 4, 4],
    [1, 1, 2, 2, 3, 4, 5, 5],
    [0, 1, 2, 2, 3, 4, 5, 5],
    [1, 2, 3, 3, 4, 5, 6, 6],
    [0, 0, 0, 1, 2, 3, 4, 4],
    [1, 1, 1, 2, 3, 4, 5, 5],
    [0, 1, 1, 2, 3, 4, 5, 5],
    [1, 2, 2, 3, 4, 5, 6, 6],
    [0, 0, 1, 2, 3, 4, 5, 5],
    [1, 1, 2, 3, 4, 5, 6, 6],
    [0, 1, 2, 3, 4, 5, 6, 6],
    [1, 2, 3, 4, 5, 6, 7, 7],
    [0, 0, 0, 0, 0, 0, 0, 1],
    [1, 1, 1, 1, 1, 1, 1, 2],
    [0, 1, 1, 1, 1, 1, 1, 2],
    [1, 2, 2, 2, 2, 2, 2, 3],
    [0, 0, 1, 1, 1, 1, 1, 2],
    [1, 1, 2, 2, 2, 2, 2, 3],
    [0, 1, 2, 2, 2, 2, 2, 3],
    [1, 2, 3, 3, 3, 3, 3, 4],
    [0, 0, 0, 1, 1, 1, 1, 2],
    [1, 1, 1, 2, 2, 2, 2, 3],
    [0, 1, 1, 2, 2, 2, 2, 3],
    [1, 2, 2, 3, 3, 3, 3, 4],
    [0, 0, 1, 2, 2, 2, 2, 3],
    [1, 1, 2, 3, 3, 3, 3, 4],
    [0, 1, 2, 3, 3, 3, 3, 4],
    [1, 2, 3, 4, 4, 4, 4, 5],
    [0, 0, 0, 0, 1, 1, 1, 2],
    [1, 1, 1, 1, 2, 2, 2, 3],
    [0, 1, 1, 1, 2, 2, 2, 3],
    [1, 2, 2, 2, 3, 3, 3, 4],
    [0, 0, 1, 1, 2, 2, 2, 3],
    [1, 1, 2, 2, 3, 3, 3, 4],
    [0, 1, 2, 2, 3, 3, 3, 4],
    [1, 2, 3, 3, 4, 4, 4, 5],
    [0, 0, 0, 1, 2, 2, 2, 3],
    [1, 1, 1, 2, 3, 3, 3, 4],
    [0, 1, 1, 2, 3, 3, 3, 4],
    [1, 2, 2, 3, 4, 4, 4, 5],
    [0, 0, 1, 2, 3, 3, 3, 4],
    [1, 1, 2, 3, 4, 4, 4, 5],
    [0, 1, 2, 3, 4, 4, 4, 5],
    [1, 2, 3, 4, 5, 5, 5, 6],
    [0, 0, 0, 0, 0, 1, 1, 2],
    [1, 1, 1, 1, 1, 2, 2, 3],
    [0, 1, 1, 1, 1, 2, 2, 3],
    [1, 2, 2, 2, 2, 3, 3, 4],
    [0, 0, 1, 1, 1, 2, 2, 3],
    [1, 1, 2, 2, 2, 3, 3, 4],
    [0, 1, 2, 2, 2, 3, 3, 4],
    [1, 2, 3, 3, 3, 4, 4, 5],
    [0, 0, 0, 1, 1, 2, 2, 3],
    [1, 1, 1, 2, 2, 3, 3, 4],
    [0, 1, 1, 2, 2, 3, 3, 4],
    [1, 2, 2, 3, 3, 4, 4, 5],
    [0, 0, 1, 2, 2, 3, 3, 4],
    [1, 1, 2, 3, 3, 4, 4, 5],
    [0, 1, 2, 3, 3, 4, 4, 5],
    [1, 2, 3, 4, 4, 5, 5, 6],
    [0, 0, 0, 0, 1, 2, 2, 3],
    [1, 1, 1, 1, 2, 3, 3, 4],
    [0, 1, 1, 1, 2, 3, 3, 4],
    [1, 2, 2, 2, 3, 4, 4, 5],
    [0, 0, 1, 1, 2, 3, 3, 4],
    [1, 1, 2, 2, 3, 4, 4, 5],
    [0, 1, 2, 2, 3, 4, 4, 5],
    [1, 2, 3, 3, 4, 5, 5, 6],
    [0, 0, 0, 1, 2, 3, 3, 4],
    [1, 1, 1, 2, 3, 4, 4, 5],
    [0, 1, 1, 2, 3, 4, 4, 5],
    [1, 2, 2, 3, 4, 5, 5, 6],
    [0, 0, 1, 2, 3, 4, 4, 5],
    [1, 1, 2, 3, 4, 5, 5, 6],
    [0, 1, 2, 3, 4, 5, 5, 6],
    [1, 2, 3, 4, 5, 6, 6, 7],
    [0, 0, 0, 0, 0, 0, 1, 2],
    [1, 1, 1, 1, 1, 1, 2, 3],
    [0, 1, 1, 1, 1, 1, 2, 3],
    [1, 2, 2, 2, 2, 2, 3, 4],
    [0, 0, 1, 1, 1, 1, 2, 3],
    [1, 1, 2, 2, 2, 2, 3, 4],
    [0, 1, 2, 2, 2, 2, 3, 4],
    [1, 2, 3, 3, 3, 3, 4, 5],
    [0, 0, 0, 1, 1, 1, 2, 3],
    [1, 1, 1, 2, 2, 2, 3, 4],
    [0, 1, 1, 2, 2, 2, 3, 4],
    [1, 2, 2, 3, 3, 3, 4, 5],
    [0, 0, 1, 2, 2, 2, 3, 4],
    [1, 1, 2, 3, 3, 3, 4, 5],
    [0, 1, 2, 3, 3, 3, 4, 5],
    [1, 2, 3, 4, 4, 4, 5, 6],
    [0, 0, 0, 0, 1, 1, 2, 3],
    [1, 1, 1, 1, 2, 2, 3, 4],
    [0, 1, 1, 1, 2, 2, 3, 4],
    [1, 2, 2, 2, 3, 3, 4, 5],
    [0, 0, 1, 1, 2, 2, 3, 4],
    [1, 1, 2, 2, 3, 3, 4, 5],
    [0, 1, 2, 2, 3, 3, 4, 5],
    [1, 2, 3, 3, 4, 4, 5, 6],
    [0, 0, 0, 1, 2, 2, 3, 4],
    [1, 1, 1, 2, 3, 3, 4, 5],
    [0, 1, 1, 2, 3, 3, 4, 5],
    [1, 2, 2, 3, 4, 4, 5, 6],
    [0, 0, 1, 2, 3, 3, 4, 5],
    [1, 1, 2, 3, 4, 4, 5, 6],
    [0, 1, 2, 3, 4, 4, 5, 6],
    [1, 2, 3, 4, 5, 5, 6, 7],
    [0, 0, 0, 0, 0, 1, 2, 3],
    [1, 1, 1, 1, 1, 2, 3, 4],
    [0, 1, 1, 1, 1, 2, 3, 4],
    [1, 2, 2, 2, 2, 3, 4, 5],
    [0, 0, 1, 1, 1, 2, 3, 4],
    [1, 1, 2, 2, 2, 3, 4, 5],
    [0, 1, 2, 2, 2, 3, 4, 5],
    [1, 2, 3, 3, 3, 4, 5, 6],
    [0, 0, 0, 1, 1, 2, 3, 4],
    [1, 1, 1, 2, 2, 3, 4, 5],
    [0, 1, 1, 2, 2, 3, 4, 5],
    [1, 2, 2, 3, 3, 4, 5, 6],
    [0, 0, 1, 2, 2, 3, 4, 5],
    [1, 1, 2, 3, 3, 4, 5, 6],
    [0, 1, 2, 3, 3, 4, 5, 6],
    [1, 2, 3, 4, 4, 5, 6, 7],
    [0, 0, 0, 0, 1, 2, 3, 4],
    [1, 1, 1, 1, 2, 3, 4, 5],
    [0, 1, 1, 1, 2, 3, 4, 5],
    [1, 2, 2, 2, 3, 4, 5, 6],
    [0, 0, 1, 1, 2, 3, 4, 5],
    [1, 1, 2, 2, 3, 4, 5, 6],
    [0, 1, 2, 2, 3, 4, 5, 6],
    [1, 2, 3, 3, 4, 5, 6, 7],
    [0, 0, 0, 1, 2, 3, 4, 5],
    [1, 1, 1, 2, 3, 4, 5, 6],
    [0, 1, 1, 2, 3, 4, 5, 6],
    [1, 2, 2, 3, 4, 5, 6, 7],
    [0, 0, 1, 2, 3, 4, 5, 6],
    [1, 1, 2, 3, 4, 5, 6, 7],
    [0, 1, 2, 3, 4, 5, 6, 7],
    [1, 2, 3, 4, 5, 6, 7, 8],
];
