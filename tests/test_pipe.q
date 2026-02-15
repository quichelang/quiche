def double(x: i64) -> i64:
    return x * 2

def add(x: i64, y: i64) -> i64:
    return x + y

def square(x: i64) -> i64:
    return x * x

def test_simple_pipe():
    # 5 |> double() -> double(5)
    assert (5 |> double()) == 10

def test_chain_pipe():
    # 5 |> double() |> double() -> double(double(5))
    res = 5 |> double() |> double()
    assert res == 20

def test_pipe_with_args():
    # 5 |> add(3) -> add(5, 3)
    res = 5 |> add(3)
    assert res == 8

def test_pipe_chain_mixed():
    # 3 |> double() |> add(10) -> add(double(3), 10)
    res = 3 |> double() |> add(10)
    assert res == 16

def test_pipe_three_chain():
    # 2 |> double() |> square() |> add(1) -> add(square(double(2)), 1)
    res = 2 |> double() |> square() |> add(1)
    assert res == 17

def main():
    test_simple_pipe()
    test_chain_pipe()
    test_pipe_with_args()
    test_pipe_chain_mixed()
    test_pipe_three_chain()
    print("All pipe tests passed!")
