# autobattler

A deterministic combat simulation engine for an autobattler game, written in Rust.

Teams of units with abilities, buffs, debuffs and movement fight on a tile grid. The engine resolves each fight tick by tick (60 ticks per second, up to five minutes of game time) and produces an event stream intended to drive a Godot front end. Every fight is seeded (ChaCha RNG), so any battle can be replayed exactly from its seed.

## Project status

Work in progress.

- **Working:** the Rust combat engine, which runs complete fights and is covered by unit and profiling tests.
- **Scaffold only:** the Godot front end in `godot/`, which is not yet connected to the engine.

Gameplay rule decisions and where each lives in the code are listed in [`mechanical decisions and their code locations.txt`](mechanical%20decisions%20and%20their%20code%20locations.txt).

## Optimisation log

Most of the development history is an exercise in making the simulation fast. The table below measures throughput at key commits with the project's own benchmark (`dev_tools::profiling::profile`), single-threaded, release build, 8 seconds per run on a 2-core cloud machine. Repeat runs varied by 2 to 3%.

| Commit | Change | Fights simulated per second |
| --- | --- | ---: |
| `1054c43` | Baseline: first commit with the benchmark harness | 22,300 |
| `b4c7f6c` | Custom `EventTimeline`: a sorted vector tuned for small event queues | 29,000 |
| `7db4492` | Cache of blocked tiles for movement | 32,200 |
| `73882a4` | `BlockedArena`: dedicated grid of blocked tiles for faster pathing | 52,300 |
| `0f657b6` | Event logging for the Godot interface added | 52,600 |
| `9a7a806` | Bug fix for early termination of event streams; tick timings rebalanced | 48,600 |
| `ec5d98d` | Event processing moved to `SmallVec` | 45,400 |
| `1135970` | Mutable event buffer passed through processing; same-tick events handled LIFO | 58,200 |
| `8782854` | Fixed-size buffer for ability cast targets | 59,800 |
| `be24836` | Current `main` | 62,900 |

Overall, the engine simulates about 2.8 times as many fights per second as the baseline.

Notes on reading the table:

- Game rules changed alongside the optimisations (for example the rebalance in `9a7a806`), so each row measures the workload of its own commit rather than an identical one.
- The two commits between `ec5d98d` and `1135970` (`6294bc3`, `2c1fa42`) do not compile in isolation, so they are not measured. Across that span throughput rose 28%.
- A multithreaded benchmark (`profile_threaded`) is also available.

## Building, testing and benchmarking

Requires a stable Rust toolchain.

```sh
cargo test                                   # unit tests plus timed profiling tests (about 2 minutes)
cargo run --release --bin profile_tests      # 10-second benchmark that also writes an event log
```

## License

MIT, for the Rust source. See [`LICENSE`](LICENSE).
