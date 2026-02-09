# Sudoku solver in "Clean" Quiche (Target Syntax)

def print_board(board: Vec[Vec[i32]]):
    """Pretty-print the sudoku board"""
    for i in range(9):
        if i % 3 == 0 and i != 0:
            print("-------+-------+-------")
        row_str: String = String.new()
        for j in range(9):
            if j % 3 == 0 and j != 0:
                row_str.push_str(" |")
            val: i32 = board[i][j]
            if val == 0:
                row_str.push_str(" .")
            else:
                row_str.push_str(" ")
                row_str.push_str(val.to_string())
        print(row_str)

def find_empty_location(board: Vec[Vec[i32]]) -> Vec[i32]:
    # board is mutref (handled by transformer)
    for row in range(0, 9):
        for col in range(0, 9):
            if board[row][col] == 0:
                # Explicit cast to satisfy return type Vec[i32]
                return [row, col]
    return [-1, -1]

def is_valid(board: Vec[Vec[i32]], row: i32, col: i32, num: i32) -> bool:
    # Check row
    for x in range(0, 9):
        if board[row][x] == num:
            return False

    # Check col
    for x in range(0, 9):
        if board[x][col] == num:
            return False

    # Check box
    start_row = row - row % 3
    start_col = col - col % 3
    for i in range(0, 3):
        for j in range(0, 3):
            r = i + start_row
            c = j + start_col
            if board[r][c] == num:
                return False

    return True

def solve_sudoku(board: Vec[Vec[i32]]) -> bool:
    # Implicit auto-borrow/reborrow should work now
    loc = find_empty_location(board)
    row = loc[0]
    col = loc[1]

    if row == -1:
        return True

    for num in range(1, 10):
        if is_valid(board, row, col, num):
            board[row][col] = num
            
            if solve_sudoku(board):
                return True
            
            board[row][col] = 0

    return False

def get_board() -> Vec[Vec[i32]]:
    return [
        [5, 3, 0, 0, 7, 0, 0, 0, 0],
        [6, 0, 0, 1, 9, 5, 0, 0, 0],
        [0, 9, 8, 0, 0, 0, 0, 6, 0],
        [8, 0, 0, 0, 6, 0, 0, 0, 3],
        [4, 0, 0, 8, 0, 3, 0, 0, 1],
        [7, 0, 0, 0, 2, 0, 0, 0, 6],
        [0, 6, 0, 0, 0, 0, 2, 8, 0],
        [0, 0, 0, 4, 1, 9, 0, 0, 5],
        [0, 0, 0, 0, 8, 0, 0, 7, 9]
    ]

def main():
    # 0 represents empty cells
    board: Vec[Vec[i32]] = get_board()

    if solve_sudoku(board):
        print("Solved:")
        print_board(board)
    else:
        print("No solution exists")
