# Ricochet Robots — Rust

Sólo logická hra pre Android ARM64 a počítače. Robot sa pohybuje po doske 16 × 16 až po stenu alebo ďalšieho robota. Dostaň robota požadovanej farby na cieľ v čo najmenšom počte ťahov.

## Spustenie

```powershell
cargo run
```

Klikni alebo ťukni na robota a potiahni po doske. Na počítači fungujú šípky, výber robotov cez 1–4, Z pre návrat ťahu a R pre reset.

Na obrazovke sú tri tlačidlá: Next level, Reset a Show solution. Show solution najprv obnoví štart a prehrá najkratšie riešenie. Roboty ostávajú v plných farbách bez kruhu pri prehrávaní. Ukážka zvýrazní trasu a aktuálny krok; zoznam krokov sa posúva. Jeden krok trvá 0,55 s (príprava 0,10 s, pohyb 0,30 s, prestávka 0,15 s). Počas ukážky je ručné ovládanie pozastavené. Reset obnoví hranie.

## Náročnosť

Posuvník MINIMUM hore nastavuje presný optimálny počet ťahov v rozsahu 2–16. Predvolená hodnota je 11. Po uvoľnení posuvníka sa generuje nové zadanie; Next level zachová zvolenú náročnosť.

Generátor vyberá nové štarty a cieľ na pevnej doske. IDA* s prípustným odhadom preverí menšie hĺbky pred prijatím riešenia. Verifikovaná dĺžka teda vyjadruje skutočné minimum, nie náhodnú cestu. Jednoťahové zadania sa nepoužívajú. Výpočet beží na pozadí s priebehom pokusov a dá sa nahradiť ďalším generovaním alebo zmenou posuvníka. Cieľový počet nie je vždy nájdený v limite výpočtu; vtedy sa zobrazí chyba a možno skúsiť Next level alebo nižšiu hodnotu. Hra nikdy potichu nezníži náročnosť. Vyššie hodnoty môžu na mobile trvať podstatne dlhšie.

## Android

Spusti Docker Desktop s Linux kontajnermi a v priečinku projektu:

```powershell
powershell -ExecutionPolicy Bypass -File .\build-android.ps1
```

APK: `target/android-artifacts/release/apk/robots.apk`.

Build skript vytvorí izolovanú kópiu projektu s lock formátom v3 pre starší Cargo kontajnera. Hostiteľský Cargo.lock sa nemení. Macroquad je uzamknuté na 0.4.13 pre Rust 1.79. Závislosti sa uchovávajú vo volume `ricochet-android-registry`; prvé načítanie starého Git indexu môže trvať niekoľko minút. Testovací podpis zostáva vo volume `ricochet-android-signing` pre aktualizácie tej istej inštalácie.

Natívny cieľ `robots` sa zhoduje s poslednou časťou package `sk.ricochet.robots`; skript kontroluje Java loader a `lib/arm64-v8a/librobots.so` v APK. Balík má vývojový podpis na lokálne testovanie. iOS zatiaľ nie je nakonfigurovaný.

## Overenie a meranie generátora

```powershell
cargo test --release --bin robots
cargo clippy --all-targets -- -D warnings
cargo run --release --example generation_benchmark -- 11 0
```

Posledný príkaz prijíma náročnosť a číslo kola. Testy porovnávajú solver s nezávislým BFS na menších zadaniach, kontrolujú 11-ťahové zadanie vrátane vylúčenia riešenia do 10 ťahov, pohyb, steny, prehrávanie a transformáciu dotykov.

