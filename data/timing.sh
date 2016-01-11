# output file
FILE=timing.csv

# test runner
TESTER="cargo run --release -- test "

for start in 100 200 300 400 500 600 700 800 900; do
  for ver in "-r" "-a"; do 
    for samp in 1 2 3 4 5 6 7; do
      echo "Running: $ver @ $start, samp:$samp"
      $TESTER $ver -s $start --outfile $FILE
    done
  done
done
