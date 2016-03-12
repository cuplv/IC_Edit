unset ADAPTON_STRUCTURAL
unset ADAPTON_NO_PURITY
cargo run --release -- windowless -s 0 -c 200000 --sparse 1 -a -f dist.csv --samp 11 -t U01G00 -d 40 20 20 20 0 0 0 0 0 0 0
cargo run --release -- windowless -s 0 -c 200000 --sparse 1 -a -f dist.csv --samp 11 -t U01G04 -d 39 19 19 19 4 0 0 0 0 0 0
cargo run --release -- windowless -s 0 -c 200000 --sparse 1 -a -f dist.csv --samp 11 -t U01G32 -d 32 12 12 12 32 0 0 0 0 0 0
cargo run --release -- windowless -s 0 -c 200000 --sparse 1 -a -f dist.csv --samp 11 -t U04G00 -u 4 -p 10 -d 40 20 20 20 0 0 0 0 0 0 0
cargo run --release -- windowless -s 0 -c 200000 --sparse 1 -a -f dist.csv --samp 11 -t U04G04 -u 4 -p 10 -d 39 19 19 19 4 0 0 0 0 0 0
cargo run --release -- windowless -s 0 -c 200000 --sparse 1 -a -f dist.csv --samp 11 -t U04G32 -u 4 -p 10 -d 32 12 12 12 32 0 0 0 0 0 0
cargo run --release -- windowless -s 0 -c 200000 --sparse 1 -a -f dist.csv --samp 11 -t U32G00 -u 32 -p 10 -d 40 20 20 20 0 0 0 0 0 0 0
cargo run --release -- windowless -s 0 -c 200000 --sparse 1 -a -f dist.csv --samp 11 -t U32G04 -u 32 -p 10 -d 39 19 19 19 4 0 0 0 0 0 0
cargo run --release -- windowless -s 0 -c 200000 --sparse 1 -a -f dist.csv --samp 11 -t U32G32 -u 32 -p 10 -d 32 12 12 12 32 0 0 0 0 0 0
cargo run --release -- windowless -s 0 -c 200000 --sparse 1 -a -f dist.csv --samp 11 -t U01G00U -d 40 20 20 20 0 0 0 0 0 20 30
cargo run --release -- windowless -s 0 -c 200000 --sparse 1 -a -f dist.csv --samp 11 -t U04G04U -u 4 -p 10 -d 39 19 19 19 4 0 0 0 0 20 30
cargo run --release -- windowless -s 0 -c 200000 --sparse 1 -a -f dist.csv --samp 11 -t U32G32U -u 32 -p 10 -d 32 12 12 12 32 0 0 0 0 20 30

cargo run --release -- windowless -s 0 -c 200000 --sparse 1 -a -f resp.csv --samp 11 -t NPS1 -u 4 -p 10 -d 39 19 19 19 4 0 0 0 0 20 30
cargo run --release -- windowless -s 0 -c 200000 --sparse 4 -a -f resp.csv --samp 11 -t NPS4 -u 4 -p 10 -d 39 19 19 19 4 0 0 0 0 20 30
cargo run --release -- windowless -s 0 -c 200000 --sparse 8 -a -f resp.csv --samp 11 -t NPS8 -u 4 -p 10 -d 39 19 19 19 4 0 0 0 0 20 30
export ADAPTON_STRUCTURAL=1
cargo run --release -- windowless -s 0 -c 200000 --sparse 1 -a -f resp.csv --samp 11 -t SPS1 -u 4 -p 10 -d 39 19 19 19 4 0 0 0 0 20 30
cargo run --release -- windowless -s 0 -c 200000 --sparse 4 -a -f resp.csv --samp 11 -t SPS4 -u 4 -p 10 -d 39 19 19 19 4 0 0 0 0 20 30
cargo run --release -- windowless -s 0 -c 200000 --sparse 8 -a -f resp.csv --samp 11 -t SPS8 -u 4 -p 10 -d 39 19 19 19 4 0 0 0 0 20 30
export ADAPTON_NO_PURITY=1
cargo run --release -- windowless -s 0 -c 200000 --sparse 1 -a -f resp.csv --samp 11 -t SIS1 -u 4 -p 10 -d 39 19 19 19 4 0 0 0 0 20 30
cargo run --release -- windowless -s 0 -c 200000 --sparse 4 -a -f resp.csv --samp 11 -t SIS4 -u 4 -p 10 -d 39 19 19 19 4 0 0 0 0 20 30
cargo run --release -- windowless -s 0 -c 200000 --sparse 8 -a -f resp.csv --samp 11 -t SIS8 -u 4 -p 10 -d 39 19 19 19 4 0 0 0 0 20 30
unset ADAPTON_STRUCTURAL
cargo run --release -- windowless -s 0 -c 200000 --sparse 1 -a -f resp.csv --samp 11 -t NIS1 -u 4 -p 10 -d 39 19 19 19 4 0 0 0 0 20 30
cargo run --release -- windowless -s 0 -c 200000 --sparse 4 -a -f resp.csv --samp 11 -t NIS4 -u 4 -p 10 -d 39 19 19 19 4 0 0 0 0 20 30
cargo run --release -- windowless -s 0 -c 200000 --sparse 8 -a -f resp.csv --samp 11 -t NIS8 -u 4 -p 10 -d 39 19 19 19 4 0 0 0 0 20 30
unset ADAPTON_NO_PURITY
