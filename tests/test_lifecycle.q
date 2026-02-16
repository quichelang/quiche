type Session:
    name: str
    ticks: i64
    events: Vec[str]

    def record(self, event: str):
        self.events.push(event)
        self.ticks += 1
        print(self.ticks)

    def advance(self, event: str):
        self.record(event)

type Engine:
    session: Session
    ok: bool

    def attach_boot_info(self, version: str):
        self.session.record(f"boot:{version}")
        self.ok = True

def test_repeated_session_transfer():
    print("Running test_repeated_session_transfer...")
    s = Session(name="worker-A", ticks=0, events=[])
    s.advance("start")
    print("Session after start:", s.ticks, s.events)
    s.advance("load-config")
    print("Session after load-config:", s.ticks, s.events)
    s.advance("ready")
    print("Session after ready:", s.ticks, s.events)

    assert s.ticks == 3
    assert (s.events |> List.len()) == 3
    assert s.events[0] == "start"
    assert s.events[1] == "load-config"
    assert s.events[2] == "ready"

def test_nested_lifecycle_updates():
    print("Running test_nested_lifecycle_updates...")
    engine = Engine(
        session=Session(name="engine-core", ticks=0, events=[]),
        ok=False
    )

    engine.attach_boot_info("v1")
    engine.session.record("sync")
    engine.session.record("serve")

    assert engine.ok == True
    assert engine.session.ticks == 3
    assert (engine.session.events |> List.len()) == 3
    assert engine.session.events[0] == "boot:v1"
    assert engine.session.events[2] == "serve"

def test_string_list_lifecycle_flow():
    print("Running test_string_list_lifecycle_flow...")
    words = ["alpha", "beta", "gamma"]

    lengths: Vec[i64] = []
    for word in words:
        lengths.push(word.len())

    assert (lengths |> List.len()) == 3
    assert lengths[0] == 5
    assert lengths[1] == 4
    assert lengths[2] == 5

def main():
    print("=== Lifecycle Suite ===")
    test_repeated_session_transfer()
    test_nested_lifecycle_updates()
    test_string_list_lifecycle_flow()
    print("=== Lifecycle Suite Passed ===")
