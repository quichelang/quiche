# Sudoku solver in "Clean" Quiche (Target Syntax)

type SudokuBoard:
    board: Vec[Vec[u8]]

    def print_board(self):
        """Pretty-print the sudoku board"""
        for i in range(9):
            if i % 3 == 0 and i != 0:
                print("-------+-------+-------")
            row_str = []
            for j in range(9):
                if j % 3 == 0 and j != 0:
                    row_str |> List.push(" |")
                val: i32 = self.board[i][j]
                if val == 0:
                    row_str |> List.push(" .")
                else:
                    row_str |> List.push(" ")
                    row_str |> List.push(str(val))
            print(row_str)

    def find_empty_location(self) -> Vec[i32]:
        board = self.board
        for row in range(0, 9):
            for col in range(0, 9):
                if self.board[row][col] == 0:
                    # Explicit cast to satisfy return type Vec[i32]
                    return [row, col]
        return [-1, -1]

    def is_valid(self, row: i32, col: i32, num: i32) -> bool:
        # Check row
        for x in range(0, 9):
            if self.board[row][x] == num:
                return False

        # Check col
        for x in range(0, 9):
            if self.board[x][col] == num:
                return False

        # Check box
        start_row = row - row % 3
        start_col = col - col % 3
        for i in range(0, 3):
            for j in range(0, 3):
                r = i + start_row
                c = j + start_col
                if self.board[r][c] == num:
                    return False

        return True

    def solve_sudoku(self) -> bool:
        # Implicit auto-borrow/reborrow should work now
        loc = self.find_empty_location()
        row = loc[0]
        col = loc[1]

        if row == -1:
            return True

        for num in range(1, 10):
            if self.is_valid(row, col, num):
                self.board[row][col] = num
                
                if self.solve_sudoku():
                    return True
                
                self.board[row][col] = 0

        return False

def get_board() -> SudokuBoard:
    return SudokuBoard([
        [5, 3, 0, 0, 7, 0, 0, 0, 0],
        [6, 0, 0, 1, 9, 5, 0, 0, 0],
        [0, 9, 8, 0, 0, 0, 0, 6, 0],
        [8, 0, 0, 0, 6, 0, 0, 0, 3],
        [4, 0, 0, 8, 0, 3, 0, 0, 1],
        [7, 0, 0, 0, 2, 0, 0, 0, 6],
        [0, 6, 0, 0, 0, 0, 2, 8, 0],
        [0, 0, 0, 4, 1, 9, 0, 0, 5],
        [0, 0, 0, 0, 8, 0, 0, 7, 9]
    ])

def main():
    # 0 represents empty cells
    sudoku: SudokuBoard = get_board()

    if sudoku.solve_sudoku():
        print("Solved:")
        sudoku.print_board()
    else:
        print("No solution exists")
