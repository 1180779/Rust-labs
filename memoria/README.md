# Projekt Memoria

## Opis

Podprojekt `memoria` implementuje prostą bazę danych w pamięci. Umożliwia tworzenie tabel, wstawianie, wybieranie i usuwanie rekordów. 
Obsługuje również zapisywanie historii wykonanych poleceń do pliku oraz odczytywanie i wykonywanie poleceń z pliku.

## Struktura Projektu

Struktura projektu (w zakresie plików) jest następująca:

```
.
├── .cargo
│   └── config.toml
├── src
│   ├── gramma.pest
│   ├── lib.rs
│   ├── main.rs
│   ├── parser.rs
│   └── query.rs
├── test
│   ├── hott.txt
│   └── hott_more.txt
├── ADDITIONS
├── Cargo.lock
├── Cargo.toml
├── clippy.toml
└── README.md
```

### Opis plików w `src`:
*   `lib.rs`: Główny plik biblioteki, który zawiera logikę bazy danych, w tym struktury `Database`, `Table`, `Record` oraz implementacje poleceń. 
*   `main.rs`: Główny plik wykonywalny, uruchamiający CLI do wprowadzania poleceń.
*   `query.rs`: Moduł odpowiedzialny za definicję struktur zapytań, takich jak `SelectQuery`, `InsertQuery`, `CreateQuery` itp.
*   `parser.rs`: Moduł zawierający logikę parsera. Wykorzystuje `pest` do parsowania zapytań w formacie tekstowym na struktury zdefiniowane w `query.rs`.
*   `gramma.pest`: Plik gramatyki dla `pest`, który definiuje składnię języka zapytań.

### Przebieg działania programu:
1. Użytkownik wprowadza polecenie w CLI (pusta linia kończy).
2. `main.rs` przekazuje polecenie do `parser.rs`, który używa `pest`a do parsowania tekstu na strukturę zapytania.
3. Struktura zapytania jest przekazywana do `lib.rs`, gdzie zapytanie jest opakowywane w odpowiednią strukturę
    `Command` zawierającą potrzebne referencje do bazy danych albo tabeli.
4. Polecenie jest wykonywane na bazie danych, a wynik jest zwracany do użytkownika.
5. Powrót do kroku 1.

### Inne pliki:
*   `clippy.toml`: Plik konfiguracyjny dla Clippy (ustawienie dopuszczalnej długości funkcji na 30 linii).

## Funkcjonalności nieopisane w poleceniu:
*   **Wybór wszystkich kolumn**: Możliwość użycia `*` w zapytaniu `SELECT`, aby wybrać wszystkie kolumny z tabeli.

## Ulubiony moduł
Mój ulubiony moduł to `query.rs`, ponieważ definiuje struktury w sposób umożliwiający posiadanie lub pożyczenie danych 
bez duplikowania kodu dla różnych wariantów struktur. 
