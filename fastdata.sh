
rm fastdist.csv
cargo run --release -- windowless -s 0 -c 80000 -a -f fastdist.csv -t NC -n
cargo run --release -- windowless -s 0 -c 40000 -a -f fastdist.csv -t U2 -u 2 -p 200
cargo run --release -- windowless -s 0 -c 40000 -a -f fastdist.csv -t U3 -u 3 -p 200
cargo run --release -- windowless -s 0 -c 40000 -a -f fastdist.csv -t U4 -u 4 -p 200
cargo run --release -- windowless -s 0 -c 80000 -a -f fastdist.csv -t ST
#cargo run --release -- windowless -s 0 -c 80000 -a -f fastdist.csv -t HC -d 10 10 10 10 50 50 50 30 10 10
#cargo run --release -- windowless -s 0 -c 80000 -a -f fastdist.csv -t HU -d 20 10 10 10 3 1 1 1 35 35 -n

echo "Dont forget to rename fastdist.csv to add the current git hashs!!"
