def main():
    # Test File.exists
    exists = File.exists("Cargo.toml")
    assert exists

    no_exist = File.exists("nonexistent_xyz.txt")
    assert no_exist == False

    # Test File.write + File.read roundtrip
    File.write("/tmp/quiche_stdlib_test.txt", "hello from quiche!")
    readback = File.read("/tmp/quiche_stdlib_test.txt")
    assert readback == "hello from quiche!"

    # Test pipe: File.read
    content = "/tmp/quiche_stdlib_test.txt" |> File.read()
    assert content == "hello from quiche!"

    # Test Path.join with pipe
    full = "src" |> Path.join("main.q")
    print("Joined:", full)

    # Test Path.basename + Path.extname
    name = "/a/b/foo.q" |> Path.basename()
    assert name == "foo.q"

    ext = "foo.q" |> Path.extname()
    assert ext == ".q"

    # Test System.cwd
    cwd = System.cwd()
    print("CWD:", cwd)

    # Cleanup
    File.rm("/tmp/quiche_stdlib_test.txt")
    assert File.exists("/tmp/quiche_stdlib_test.txt") == False

    print("All stdlib tests passed!")
