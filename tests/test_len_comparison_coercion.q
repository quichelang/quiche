def test_len_comparison_with_i64_variable():
    print("Running test_len_comparison_with_i64_variable...")
    s = "alpha"
    target: i64 = 5

    assert s.len() == target
    assert len(s) == target


def test_len_comparison_mixed_forms():
    print("Running test_len_comparison_mixed_forms...")
    words = ["a", "bb", "ccc"]
    expected: i64 = 3

    assert words[2].len() == expected
    assert len(words[2]) == expected


def main():
    print("=== Len Comparison Coercion Suite ===")
    test_len_comparison_with_i64_variable()
    test_len_comparison_mixed_forms()
    print("=== Len Comparison Coercion Suite Passed ===")
