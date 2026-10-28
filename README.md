# Ricochet Robots — Rust

Hrateľná logická hra pre jedného hráča, pripravená pre Android (na výšku), s rovnakým Rust kódom pre Windows, Linux a macOS. Používa Macroquad a vlastnú dosku; nepotrebuje obrázky ani internet počas hrania.

## Spustenie na počítači

Nainštaluj stabilný Rust a spusti v tomto priečinku:

```powershell
cargo run --release
```

## Ovládanie

Ťukni na robota a potiahni po doske v požadovanom smere. Na obrazovke sú iba tlačidlá Next level, Reset a Show solution. Na počítači fungujú šípky, čísla 1–4 na výber robota, Z na návrat a R na reštart.

Robot ide až po stenu alebo iného robota. Dostaň robota požadovanej farby na kruhový cieľ s čo najmenším počtom ťahov. Neúspešný pohyb sa nepočíta. Reset obnoví začiatok kola; Next level otvorí ďalšie zadanie; Show solution obnoví kolo a prehrá najkratšie riešenie spolu so zoznamom krokov.

Hra má 16 deterministických kombinácií štartov a cieľového robota, ktoré sa opakujú. Zadania vytvára BFS do hĺbky štyroch ťahov. Náročnosť cieľa určuje jeho prvé dosiahnutie naprieč všetkými zostavami robotov. Ciele dostupné za nula alebo jeden ťah sú vylúčené; uložené riešenie je najkratšie. Po dokončení zobrazuje optimálny počet ťahov. Ide o sólový tréningový režim, bez súťažného časovača a multiplayera.

## Android APK cez Docker

Spusti Docker Desktop s Linux kontajnermi. V PowerShelli v koreňovom priečinku projektu:

```powershell
powershell -ExecutionPolicy Bypass -File .\build-android.ps1
```

APK vznikne v `target/android-artifacts/release/apk/`. Tento balík používa vývojový podpis na lokálne testovanie. Vydanie v obchode vyžaduje vlastný podpis a kontrolu aktuálnych požiadaviek obchodu. Konfigurácia cieli na ARM64 Android telefóny. Zostavenie ARM64 APK bolo overené v Dockeri. Spustenie a dotyky na fyzickom telefóne zatiaľ neboli overené.

Oficiálny postup: https://macroquad.rs/articles/android/

iOS balík zatiaľ nie je nakonfigurovaný.

## Overenie

```powershell
cargo test
cargo check
cargo fmt --check
```

Testy kontrolujú zastavenie pri robotoch a hraniciach, symetriu stien, riešiteľnosť všetkých 16 zadaní, návrat ťahu a reštart. Nezávislé prehľadanie všetkých kratších postupností overuje minimálny počet ťahov aj vylúčenie jednoťahových zadaní.

### Kompatibilita Cargo.lock

Docker obraz obsahuje starší build nástroj, ktorý nečíta lock formát v4. Namiesto priameho Docker príkazu preto použi:

```powershell
powershell -ExecutionPolicy Bypass -File .\build-android.ps1
```

Skript skopíruje zdrojové súbory do kontajnera a iba v tejto kópii nastaví lock formát v3. Uzamknuté verzie závislostí ostávajú rovnaké a hostiteľský Cargo.lock sa nemení. Výstupy sa zapíšu do lokálneho priečinka target. Tento postup bol overený kompletným zostavením a podpísaním APK.

Pre Docker s Rustom 1.79 je Macroquad uzamknuté na 0.4.13 (s fontdue 0.7.3). Novšie verzie Macroquad a fontdue vyžadujú novšie Rust API. Build skript uchováva stiahnuté závislosti v Docker volume `ricochet-android-registry` pre rýchlejšie opakované zostavenia.




### Nativna kniznica a instalacia

Rust binarny ciel sa vola `robots`, rovnako ako posledna cast Android package `sk.ricochet.robots`. Tento build nastroj z nej odvodzuje meno pre System.loadLibrary. APK sa preto vola `robots.apk` a obsahuje `lib/arm64-v8a/librobots.so`. Build skript overuje zhodu Java loadera s obsahom APK.

Lokalny testovaci podpis je ulozeny v Docker volume `ricochet-android-signing`, aby dalsie buildy mohli aktualizovat tu istu instalaciu. Pri prechode zo stareho APK, ktore malo jednorazovy podpis, povodnu aplikaciu najprv odinstaluj. Tym sa odstrania aj jej lokalne data.


