rec-demo:
  cargo build # cache build for seemless recording
  asciinema rec --command "cargo run -q -- -d 5s 1s -N -t 10 -c 2" tmp.cast
  agg --font-family "SpaceMono Nerd Font" --font-size 28 --theme github-dark tmp.cast assets/demo.gif
  rm tmp.cast
