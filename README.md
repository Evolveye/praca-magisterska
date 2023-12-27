# Praca magisterska

## Profilowanie

W celu wykonania pomiaru należy zastosować następujace polecenia:

1. `cargo build`, aby zbudować aplikację. Należy zbudować ją w środowisku linuxa/WSL2
2. `valgrind --tool=massif ./target/...`, aby wykonać pomiar
3. `ms_print <raport narzędzia valgrind>`, aby wyświetlić sformatowane dane, w tym wykres
