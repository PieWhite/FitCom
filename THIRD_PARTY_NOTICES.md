# Third-party notices

FitCom project code is GPL-3.0-only. External libraries and assets retain their own
licenses. `Cargo.lock` identifies exact Rust packages. The combined
[license bundle](THIRD_PARTY_LICENSES.txt) preserves notices for the locked Windows
x64 and Apple Silicon macOS graphs, including build/test packages in the metadata,
native Opus and the bundled fonts. It combines packaged files with notices from
exact upstream source commits. Distributors must preserve notices and supply
corresponding source as required by applicable licenses.

## Fonts

- Inter: SIL Open Font License 1.1; `crates/app/src/ui/assets/fonts/Inter-OFL.txt`.
- Instrument Serif: SIL Open Font License 1.1; `crates/app/src/ui/assets/fonts/InstrumentSerif-OFL.txt`.
- JetBrains Mono: SIL Open Font License 1.1; `crates/app/src/ui/assets/fonts/JetBrainsMono-OFL.txt`.

## Libraries and native components

FitCom uses GPUI/gpui-kit, Opus, SQLite, rustls, Quinn, ring, and other packages.
Font licensing is separate from the code license. `cargo deny check -W unmaintained` checks declared
package licenses and advisories, not provenance of copied assets or data.

Nine packages declare MIT and/or Apache-2.0 licensing but do not provide a
standalone upstream copyright/license notice: `mac` 0.1.1, `realfft` 3.5.0,
`seahash` 4.1.0, `simd_helpers` 0.1.0, `block` 0.1.6, `leak` 0.1.2,
`leaky-cow` 0.1.1, `malloc_buf` 0.0.6 and `objc_exception` 0.1.2. The bundle
retains their published declarations, attribution and canonical license text;
missing package-specific copyright wording has not been invented. This records
notice-provenance limits rather than asserting that those packages are unlicensed.

## Wordle data: unresolved provenance

`crates/app/src/wordle_woorden.txt` was copied from a published Wordle client word
list. No redistribution license has been established for that copy. It remains
unchanged at the owner's explicit request. Its inclusion does not assert ownership
or a right to relicense that data under GPL. This remains an unresolved distribution
risk. Daily puzzles use an undocumented NYT endpoint that can become unavailable.
FitCom is not affiliated with or endorsed by The New York Times. Wordle is a
third-party name. YouTube/Spotify integrations do not imply affiliation or endorsement.
