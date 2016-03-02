
rm fastdist.csv
cargo run --release -- windowless -s 0 -c 50000 -a -f fastdist.csv
sed 's/Fast,/Fast-ST,/g' fastdist.csv > fastdist2.csv
mv fastdist2.csv fastdist.csv
cargo run --release -- windowless -s 0 -c 50000 -a -f fastdist.csv -n
sed 's/Fast,/Fast-NC,/g' fastdist.csv > fastdist2.csv
mv fastdist2.csv fastdist.csv
cargo run --release -- windowless -s 0 -c 50000 -a -f fastdist.csv -d 10 10 10 10 50 50 50 30 10 10
sed 's/Fast,/Fast-HC,/g' fastdist.csv > fastdist2.csv
mv fastdist2.csv fastdist.csv
cargo run --release -- windowless -s 0 -c 50000 -a -f fastdist.csv -d 20 10 10 10 3 1 1 1 50 50
sed 's/Fast,/Fast-HU,/g' fastdist.csv > fastdist2.csv
mv fastdist2.csv fastdist.csv
echo "Dont forget to rename fastdist.csv to add the current git hashs!!"
