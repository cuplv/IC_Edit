export RUST_MIN_STACK = 10485760

# output file
FILE=timing.csv

# test runner
TESTER="cargo run --release -- test -n"

for start in 1000 2000 3000 4000 5000 6000 7000 8000 9000; do
  for ver in "-r" "-a"; do 
    for samp in 1 2 3 4 5 6 7; do
      echo "Running: $ver @ $start, samp:$samp"
      $TESTER $ver -s $start --outfile $FILE
    done
  done
done
