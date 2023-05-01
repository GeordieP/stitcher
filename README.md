# stitcher

a little ffmpeg utility i needed

- give it an `--input_path` - directory containing a bunch of wav or mp3 files
- optionally give it an `--out` - output file name (defaults to current date)
- the tool will run ffmpeg and stitch the files together

> NOTE: the tool will look for an ffmpeg binary in your system $PATH, or in `./vendor/ffmpeg/ffmpeg`. see the readme in `./vendor/README.md` for more info

---

needs rust nightly - uses `#![feature()]`

run

```
cargo run -- -i ./test/stitcher/sounds/wav/ -o my_test_output.wav
```


test

```
cargo test
```
