# cphaze

Build and run the visualizer:
```
cargo run --release
```
To automatically recompile and reload the test code, in another terminal:
```
cargo watch -w lib -x 'build -p lib --release'
```
Edit that test code in
```
./lib/src/lib.rs
```
