cargo build
valgrind --tool=massif --massif-out-file=massif.out ./target/debug/praca-magisterska
ms_print massif.out | grep -v -- '->'
