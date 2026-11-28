# Changelog

## [0.12.0](https://github.com/klercke/ProtoBot/compare/v0.11.0...v0.12.0) (2026-11-28)


### Features

* **santa:** add a budget and rules link to secret santa events ([#147](https://github.com/klercke/ProtoBot/issues/147)) ([333e637](https://github.com/klercke/ProtoBot/commit/333e6370905452f71fff82a177899b0a2ca2221c)), closes [#119](https://github.com/klercke/ProtoBot/issues/119)
* **santa:** announce automatic draw results in the event's channel ([2fc05e8](https://github.com/klercke/ProtoBot/commit/2fc05e8fd2c6b9041a756f8f63741d08315cd445))
* **santa:** block registration changes after the draw ([#137](https://github.com/klercke/ProtoBot/issues/137)) ([fb5c9af](https://github.com/klercke/ProtoBot/commit/fb5c9af7560523137411b7cc785d5f784f23ef86)), closes [#114](https://github.com/klercke/ProtoBot/issues/114)
* **santa:** draw automatically at draw time ([2fc05e8](https://github.com/klercke/ProtoBot/commit/2fc05e8fd2c6b9041a756f8f63741d08315cd445))
* **santa:** give secret santa participants a role ([#152](https://github.com/klercke/ProtoBot/issues/152)) ([b662752](https://github.com/klercke/ProtoBot/commit/b662752e367867c0d9340e3bc62cbbc5011e6817)), closes [#143](https://github.com/klercke/ProtoBot/issues/143)
* **santa:** remind santas before gift time ([9c10a8f](https://github.com/klercke/ProtoBot/commit/9c10a8f8c23e65a34609d85dedbe18bd7229e912))
* **santa:** report failed assignment dms ([#140](https://github.com/klercke/ProtoBot/issues/140)) ([00e5ea9](https://github.com/klercke/ProtoBot/commit/00e5ea9f703734b881c547fef1bc2c95e6266036)), closes [#116](https://github.com/klercke/ProtoBot/issues/116)
* **santa:** require force to reshuffle ([#138](https://github.com/klercke/ProtoBot/issues/138)) ([f68aacd](https://github.com/klercke/ProtoBot/commit/f68aacd759bbf9c2e2aad67bdcade665618152dc)), closes [#115](https://github.com/klercke/ProtoBot/issues/115)
* **santa:** require manage events for admin commands ([#133](https://github.com/klercke/ProtoBot/issues/133)) ([cd2ecc1](https://github.com/klercke/ProtoBot/commit/cd2ecc11901d1e3acf234bf1285161d57df20cf5)), closes [#64](https://github.com/klercke/ProtoBot/issues/64)
* **santa:** show users their giftee on request ([02e9de6](https://github.com/klercke/ProtoBot/commit/02e9de6b34b3ae39dd650566c0a0cca0fa0852c7))


### Bug Fixes

* **santa:** build wishlist links correctly when a steam url ends in a slash ([02e9de6](https://github.com/klercke/ProtoBot/commit/02e9de6b34b3ae39dd650566c0a0cca0fa0852c7))
* **santa:** explain out-of-range timestamps in /santa_set_time instead of failing ([9c10a8f](https://github.com/klercke/ProtoBot/commit/9c10a8f8c23e65a34609d85dedbe18bd7229e912))
* **santa:** reference santa_guilds.guild id from participants ([#136](https://github.com/klercke/ProtoBot/issues/136)) ([ca64847](https://github.com/klercke/ProtoBot/commit/ca64847c32f2053f5b257829d19c3c9ef7611ae4)), closes [#135](https://github.com/klercke/ProtoBot/issues/135)
* stop logging an error on startup when the logs directory already exists ([2fc05e8](https://github.com/klercke/ProtoBot/commit/2fc05e8fd2c6b9041a756f8f63741d08315cd445))


### Documentation

* **readme:** explain prod vs preview ([#153](https://github.com/klercke/ProtoBot/issues/153)) ([4352ce7](https://github.com/klercke/ProtoBot/commit/4352ce79359c7dc8cf5125ae0ada5e747bf2654d))

## [0.11.0](https://github.com/klercke/ProtoBot/compare/v0.10.1...v0.11.0) (2026-11-25)


* release 0.11.0 ([#129](https://github.com/klercke/ProtoBot/issues/129)) ([a07530b](https://github.com/klercke/ProtoBot/commit/a07530bb23f7c12adfec2a752c0683ad5f84224b))


### Documentation

* add new logo ([#110](https://github.com/klercke/ProtoBot/issues/110)) ([0278d96](https://github.com/klercke/ProtoBot/commit/0278d96b819a59862b7bf25baa494c52ac826f67))
* relicense to AGPL-3.0-or-later and adopt REUSE ([f1ba520](https://github.com/klercke/ProtoBot/commit/f1ba520bb76ee2f02c2d3ca5f32904f146604b27))

## [0.10.1](https://github.com/klercke/ProtoBot/compare/v0.10.0...v0.10.1) (2026-11-24)


### Bug Fixes

* **deps:** update or ignore rustsec advisories ([#93](https://github.com/klercke/ProtoBot/issues/93)) ([08b417f](https://github.com/klercke/ProtoBot/commit/08b417f1ce5bd6710889eb42d29af1a58ab3f19a))
* point help command docs link at GitHub ([f0d383a](https://github.com/klercke/ProtoBot/commit/f0d383aee66f70eb8630e2f4ed544b3492ada027))
* **santa:** move Discord events when santa_set_time changes times ([f0d383a](https://github.com/klercke/ProtoBot/commit/f0d383aee66f70eb8630e2f4ed544b3492ada027))
* **santa:** reject invalid timestamps before saving anything ([f0d383a](https://github.com/klercke/ProtoBot/commit/f0d383aee66f70eb8630e2f4ed544b3492ada027))
* **santa:** save draw time set with santa_set_time ([f0d383a](https://github.com/klercke/ProtoBot/commit/f0d383aee66f70eb8630e2f4ed544b3492ada027))


### Documentation

* **readme:** make readme usable ([#83](https://github.com/klercke/ProtoBot/issues/83)) ([38162b9](https://github.com/klercke/ProtoBot/commit/38162b926b61ef7dfc793b14d2caaef84fa8a70f))
