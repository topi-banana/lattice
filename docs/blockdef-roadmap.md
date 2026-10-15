# BlockDef ロードマップ — Minecraft Java Edition 26.2

`lattice/examples/all_blocks.rs` は 26.2 の全 1196 ブロックを現行ファセット（`Sticky` /
`Movable` / `UnMovable` / `InstantMinable` / `Breakable` / `Flammable` / `Power` /
`Empty`）だけで登録したものです。本ドキュメントは、そこに登録したスタブ型のうち、
vanilla の挙動を再現するために**新しい BlockDef（能力・フック・基盤）が必要なもの**を
優先度つきで整理したロードマップです。

- 対象: Minecraft Java Edition 26.2（Chaos Cubed、全 1196 ブロック）
- データ: `PrismarineJS/minecraft-data`（`pc_26_2`）、`misode/mcmeta`（`26.2-summary`）、
  Minecraft Wiki（Piston / Fire の各表）
- 判定材料: vanilla の block definition type と各ブロックの実挙動
- 対象数はカテゴリ間で重複があります（例: ドアは「レッドストーン」と「操作」の両方に数える）

## 実装可否の凡例

| 記号 | 意味 |
| --- | --- |
| ◎ | 既存フック（`on_placed` / `on_scheduled_tick` / `on_random_tick` / `on_neighbor_changed` / `TickCtx`）だけで実装可能 |
| △ | ライブラリへの API 追加（`on_use`、アイテム抽象など）が必要 |
| × | 一般エンティティ層など、BlockDef より下の基盤が必要 |

## サマリ

| 優先 | カテゴリ | 提案する能力／代表 BlockDef | 対象数 | 実装可否 |
| --- | --- | --- | ---: | --- |
| P0 | 流体 | `Fluid` + `WaterBlock` / `LavaBlock` / `BubbleColumnBlock` | 3 | ◎ |
| P0 | 落下 | `Falling` + `SandBlock` / `GravelBlock` / `ConcretePowderBlock` / `AnvilBlock` / `SuspiciousBlock` / `DragonEggBlock` / `ScaffoldingBlock` | 26 | ◎ |
| P0 | 火 | `FireBlock` / `CampfireBlock` | 4 | ◎ |
| P0 | 爆発 | `Explosive` + `TntBlock` / `RespawnAnchorBlock` | 2 | ◎ |
| P0 | 形状・接続 | `StairBlock` / `SlabBlock` / `FenceBlock` / `WallBlock` / `PaneBlock` / `RotatedPillarBlock` / `ChainBlock` | 約 310 | ◎ |
| P0 | サポート必須 | `Supported` + `TorchBlock` / `LanternBlock` / `LadderBlock` / `SugarCaneBlock` / `RailBlock` / `SnowLayerBlock` など | 約 53 | ◎ |
| P1 | レッドストーン | `SignalSource` / `SignalWire` / `SignalLogic` / `RedstoneControllable` + `RedstoneWireBlock` / `RepeaterBlock` / `ComparatorBlock` / `ObserverBlock` / `LeverBlock` / `ButtonBlock` / `PressurePlateBlock` / `DaylightDetectorBlock` / `TargetBlock` / `TripwireBlock` / `RedstoneLampBlock` / `CopperBulbBlock` / `SculkSensorBlock` / `NoteBlock` | 約 130 | ◎ |
| P1 | 成長・拡散 | `Growable` + `CropBlock` / `SaplingBlock` / `StemBlock` / `KelpBlock` / `VineBlock` / `AmethystClusterBlock` / `BuddingAmethystBlock` / `PointedDripstoneBlock` / `SulfurSpikeBlock` / `DripleafBlock` / `NyliumBlock` / `LeavesBlock` / `CoralBlock` / `EggBlock` / `SculkBlock` / `GlowLichenBlock` / `MossBlock` / `GrassSpreadBlock` | 約 120 | ◎ |
| P1 | 酸化 | `Oxidizable` + `OxidationStage`（+ ワックス） | 約 122 | ◎ |
| P2 | 操作・開閉 | `on_use` フック + `DoorBlock` / `TrapdoorBlock` / `FenceGateBlock` / `CakeBlock` / `CandleBlock` / `CauldronBlock` / `ComposterBlock` / `BellBlock` / 作業台系 / `FlowerPotBlock` | 約 100 | △ |
| P2 | 収納・ブロックエンティティ | `Container` + `ChestBlock` / `BarrelBlock` / `ShulkerBoxBlock` / `FurnaceBlock` / `HopperBlock` / `BrewingStandBlock` / `CrafterBlock` / `ShelfBlock` / `LecternBlock` / `JukeboxBlock` / `DecoratedPotBlock` / `BeehiveBlock`、看板・バナー・ベッド・頭部 | コア約 60 | △ |
| P3 | 接触効果 | `CactusBlock` / `MagmaBlock` / `PowderSnowBlock` / `CobwebBlock` / `HoneyBlock` / `SlimeBlock` / `SoulSandBlock` / `WitherRoseBlock` / `SweetBerryBushBlock` など | 約 20 | × |
| P3 | 輸送 | `RailBlock` / `PoweredRailBlock` / `DetectorRailBlock` + トロッコ | 4 | × |
| P3 | 機構・進行 | `SpawnerBlock` / `TrialSpawnerBlock` / `VaultBlock` / `EndPortalBlock` / `EndGatewayBlock` / `NetherPortalBlock` / `BeaconBlock` / `ConduitBlock` / `InfestedBlock` / `CreakingHeartBlock` / `DriedGhastBlock` / `StructureBlock` / `CommandBlock` | 約 30 | × |

---

## P0 — 既存フックで書ける「核」

### 流体（Fluid）— 3 ブロック

- 対象: `water`, `lava`, `bubble_column`
- 提案: `Fluid` ファセット（source / flowing / level）、`FluidLevel` StateCodec、
  `WaterBlock` / `LavaBlock` / `BubbleColumnBlock`
- 実装メモ: 拡散は `on_scheduled_tick` の再スケジュールで書ける。`waterlogged` 状態を持つ
  ブロックは 429 あり、流体側の対応（水没セルの扱い）もここで定義したい。

### 落下（Falling）— 26 ブロック

- 対象: `sand`, `red_sand`, `gravel`、`concrete_powder` ×16、`anvil` ×3、
  `suspicious_sand` / `suspicious_gravel`、`dragon_egg`、`scaffolding`
- 提案: `Falling` ファセット + 各 BlockDef。`TickCtx::move_cell` 相当で 1gt ごとに落下し、
  着地で通常ブロックに戻す
- 実装メモ: 見た目の移動アニメーション（MovingBlock entity）は別途。26.2 の block
  definition では `sand` / `colored_falling` / `concrete_powder` / `anvil` / `brushable` /
  `dragon_egg` / `scaffolding` の 7 type が相当する。

### 火（Fire）— 4 ブロック

- 対象: `fire`, `soul_fire`, `campfire`, `soul_campfire`
- 提案: `FireBlock` / `CampfireBlock`。延焼・消火・点火のロジック
- 実装メモ: `Flammable` マーカーは既にある。延焼確率・燃え尽きは `on_random_tick` /
  `on_scheduled_tick`、`age` は State で持てる（`FireState` は `all_blocks.rs` に定義済み）。

### 爆発（Explosive）— 2 ブロック

- 対象: `tnt`（点火 → 遅延 → 爆発）、`respawn_anchor`（チャージ・爆発）
- 提案: `Explosive` ファセット + `TntBlock` / `RespawnAnchorBlock`
- 実装メモ: 半径破壊は `TickCtx::remove` のループで書ける。`World::explode` のような
  プリミティブを用意すると再利用しやすい。なおベッドの Nether 爆発も同カテゴリ。

### 形状・接続（Shape / Connection）— 約 310 ブロック

- 対象（ファミリ内訳は[付録 B](#付録-b-形状ファミリの内訳)）: 階段・ハーフブロック・塀・
  フェンス・ガラス窓・鉄格子・柱（`axis`）・ドア/トラップドア/フェンスゲート・ボタン/レバー・
  バナー/頭部の向き・大きなドリップリーフなど
- 提案: `StairBlock` / `SlabBlock` / `FenceBlock` / `WallBlock` / `PaneBlock` /
  `RotatedPillarBlock` / `ChainBlock` / `GlazedTerracottaBlock` など
- 実装メモ: StateCodec は `all_blocks.rs` に定義済み（`StairState`, `ConnectionState`,
  `WallState`, `PillarState` など）。あとは `on_placed` / `on_neighbor_changed` で
  shape・接続・`axis` を再計算するだけ。共通ロジックはヘルパー trait にまとめると良い。

### サポート必須（Supported）— 約 53 ブロック

- 対象: `torch` / 壁トーチ、ランタン系、`ladder`、`sugar_cane`、`cactus`、`cocoa`、
  つる・ケルプ・シーグラス、`snow`（雪層）、`lily_pad`、レール系、`tripwire`、
  サンゴ系、`sea_pickle`、アメジストの芽、キノコ・ネザー菌類・根・新芽、
  `pink_petals` / `leaf_litter` など
- 提案: `Supported` ヘルパー + `TorchBlock` / `LanternBlock` / `LadderBlock` /
  `SugarCaneBlock` / `RailBlock` / `SnowLayerBlock` など
- 実装メモ: `on_neighbor_changed` で支持を確認し、失われたら `remove_self`。
  ワールド外（`H` の上端・下端）の扱いもここで定義する。

---

## P1 — 既存フックで書ける「動的」挙動

### レッドストーン（Redstone）— 約 130 ブロック

- 電源: `lever`、ボタン 14、感圧板 16（重量感圧板含む）、赤石トーチ 2、`redstone_block`、
  日照センサー、ディテクターレール、`target`、トリップワイヤーフック、
  スカルクセンサー / 調律スカルクセンサー
- 伝導・論理: `redstone_wire`、リピーター、コンパレーター、オブザーバー、避雷針 8
- 被駆動: ドア・トラップドア・フェンスゲート **54**、赤石ランプ、銅の電球 8、
  音符ブロック、パワードレール / アクティベーターレール、ディスペンサー / ドロッパー /
  クラフター、TNT の点火
- 提案: `SignalSource` / `SignalWire` / `SignalLogic` / `RedstoneControllable` の分割、
  または単一 `Redstone` ファセット。型は `RedstoneWireBlock` / `RepeaterBlock` /
  `ComparatorBlock` / `ObserverBlock` / `LeverBlock` / `ButtonBlock` /
  `PressurePlateBlock` / `DaylightDetectorBlock` / `TargetBlock` / `TripwireBlock` /
  `RedstoneLampBlock` / `CopperBulbBlock` / `SculkSensorBlock` / `NoteBlock` /
  `LightningRodBlock` など
- 実装メモ: 伝導はスケジュール tick の再帰で書ける。quasi-connectivity・ソフトインバージョン
  などの Java 固有挙動は後回しで良い。

### 成長・拡散（Growth）— 約 120 ブロック

- 対象: 作物 7 種（`wheat` / `carrots` / `potatoes` / `beetroots` / `nether_wart` /
  `torchflower_crop` / `pitcher_crop`）、苗木 8+（`mangrove_propagule` 含む）、
  茎 4（`stem` / `attached_stem`）、`kelp` / `seagrass`、つる 7、コーラス 2、
  アメジスト（芽・塊・`budding_amethyst`）、`pointed_dripstone` / `sulfur_spike`、
  ドリップリーフ 3、`nylium`、ネザー菌類、草 / シダ / 背の高い花 / キノコ、
  サンゴ（死滅含む）、卵 3（カメ・スニッファー・カエル）、スカルク系 3、
  ヒカリゴケ、葉（枯死）、苔、`leaf_litter` / `bush` など
- 提案: `CropBlock`, `SaplingBlock`, `StemBlock`, `KelpBlock`, `VineBlock`,
  `AmethystClusterBlock`, `BuddingAmethystBlock`, `PointedDripstoneBlock`,
  `SulfurSpikeBlock`, `DripleafBlock`, `NyliumBlock`, `LeavesBlock`, `CoralBlock`,
  `EggBlock`, `SculkBlock`, `SculkVeinBlock`, `GlowLichenBlock`, `MossBlock`,
  `GrassSpreadBlock`
- 実装メモ: `on_random_tick` と `World::random_tick` が既にあるため、そのまま使える。
  `age` 系の StateCodec は `all_blocks.rs` に定義済み（`CropAgeState`, `SaplingState`,
  `LeavesState` など）。

### 酸化（Oxidation）— 約 122 ブロック

- 対象: 銅系一式（ブロック / 階段 / ハーフ / ドア / トラップドア / 電球 / チェスト /
  ゴーレム像 / グレーチング / ランタン / 鎖 / 延べ棒 / 避雷針）と、その `exposed` /
  `weathered` / `oxidized` / `waxed` バリアント
- 提案: `Oxidizable` ファセット + `OxidationStage` StateCodec、`CopperBlock` 系の型。
  ワックスがけ（ハニカム）は「操作」側
- 実装メモ: `on_random_tick` で 1 段階ずつ進める。ワックス状態は State か別 type の
  どちらでも表現できる（26.2 は別ブロック）。

---

## P2 — ライブラリ API の追加が必要

### 操作・開閉（Interactive）— 約 100 ブロック

- 対象: ドア 21 / トラップドア 21 / フェンスゲート 12、ボタン・レバー・感圧板、
  `cake`、キャンドル 34、大釜 4、`composter`、焚き火、`bell`、音符ブロック、
  日照センサー、`flower_pot` 39、`respawn_anchor`、作業台系
  （`crafting_table` / `cartography_table` / `fletching_table` / `smithing_table` /
  `stonecutter` / `loom` / `grindstone` / `enchanting_table` / 金床 3）
- 必要なもの: **`on_use` 相当のフック**。`TickCtx` のようにセルを読み書きできる
  `UseCtx`（使用者・手・アイテムの情報つき）を追加する
- 提案: `DoorBlock`, `TrapdoorBlock`, `FenceGateBlock`, `CakeBlock`, `CandleBlock`,
  `CauldronBlock`, `ComposterBlock`, `BellBlock`, `FlowerPotBlock` など。
  ドア系は「レッドストーンでも開閉する」ため、信号側との連携も必要

### 収納・ブロックエンティティ（Container）— コア約 60 ブロック

- 対象（コア）: `chest` / `trapped_chest` / 銅チェスト 8 / `ender_chest` / `barrel` /
  シュルカーボックス 17 / `furnace` / `blast_furnace` / `smoker` / `hopper` /
  `dispenser` / `dropper` / `brewing_stand` / `crafter` / `chiseled_bookshelf` /
  本棚（shelf）12 / `lectern` / `jukebox` / `decorated_pot` / 焚き火 2 / 養蜂箱 2
- 対象（データ保持）: 看板 48、バナー 32、ベッド 16、頭部 13
- 必要なもの: `ItemStack` / `Inventory` 抽象（BlockEntity 上で自作は可能だが、
  保存・スタック・スロット操作の標準化が必要）
- 提案: `Container` ファセット + `ChestBlock`, `BarrelBlock`, `ShulkerBoxBlock`,
  `FurnaceBlock`, `HopperBlock`, `DispenserBlock`, `BrewingStandBlock`,
  `CrafterBlock`, `ShelfBlock`, `LecternBlock`, `JukeboxBlock`, `DecoratedPotBlock`,
  `BeehiveBlock` など。看板・バナー・ベッド・頭部は「データを保持するだけ」の
  軽量 BlockDef として先に実装できる

---

## P3 — エンティティ層など基盤が前提

### 接触効果（Contact effects）— 約 20 ブロック

- 対象: `cactus`, `magma_block`, `campfire`, `sweet_berry_bush`, `powder_snow`,
  `cobweb`, `honey_block` / `slime_block`（反発） , `soul_sand`, `bubble_column`
  （ドラッグ） , `scaffolding` / `ladder`（登攀） , `wither_rose`, `pointed_dripstone` /
  `anvil`（落下ダメージ） , `suspicious_sand` / `suspicious_gravel`（ブラッシング）
- 必要なもの: 一般エンティティ層（現状は BlockEntity のみ）。エンティティがセル侵入時に
  呼ばれるフックが必要

### 輸送（Transport）— 4 ブロック

- 対象: `rail` / `powered_rail` / `activator_rail` / `detector_rail`
- 必要なもの: トロッコ用のエンティティ層。レール単体は「形状・接続」と「信号」で
  大半が表現できる

### 機構・進行（Technical）— 約 30 ブロック

- 対象: `spawner` / `trial_spawner` / `vault` / `end_portal` / `end_gateway` /
  `nether_portal` / `beacon` / `conduit` / `respawn_anchor` / `infested` ×7 /
  `creaking_heart` / `dried_ghast` / `jigsaw` / `structure` / `structure_void` /
  `test_block` / `test_instance_block` / `barrier` / `light` / コマンドブロック 3 /
  `moving_piston` / `piston_head` / `dragon_egg`
- 提案: ゲーム進行・生成用が大半のため低優先。`infested`（破壊時にスポーン）や
  `dragon_egg` などは比較的容易

### その他の個別メカニクス

- `sponge` / `wet_sponge`: 水の吸収
- `ice` / `frosted_ice` / `snow`（雪層）: 融解・凍結
- `farmland` / `dirt_path` / `mud` / `snowy_dirt` / `mycelium` / `grass_block`:
  水分・踏み荒らし・他ブロックへの変化
- `pumpkin` / `carved_pumpkin` / `jack_o_lantern`: 彫刻・点灯
- `brown_mushroom_block` / `red_mushroom_block` / `mushroom_stem`: ハサミで分離
- `end_portal_frame`: エンダーアイの挿入
- `heavy_core`、`redstone_ore`（触れると発光）、`drop_experience`（鉱石 16 種の
  XP ドロップ）
- 26.2 新規: `potent_sulfur`（ガス・ガイザー）、`sulfur_spike`（鍾乳石ライク）、
  `dried_ghast`（水和）

---

## ライブラリ側の不足（BlockDef より下）

1. **`on_use` / `UseCtx`** — 操作系全般の前提。使用者・アイテム・手の情報つき
2. **`ItemStack` / `Inventory`** — 収納・醸造・クラフト。保存形式も含めて
3. **一般エンティティ層** — MovingBlock、落下ブロックの見た目、点火 TNT、トロッコ、
   アイテム、XP、スポーン
4. **爆発プリミティブ** — `World::explode` 相当（`TickCtx::remove` でも書けるが共通化推奨）
5. **ドロップ / ツール要件** — 現状 `break_ticks` のみ。`BlockDef::drops`、採掘道具、
   シルクタッチ・幸運
6. **流体 × `waterlogged` の連携** — 水没状態を持つ 429 ブロックの扱い
7. **`PistonBlock` の拡張** — 粘着ピストンの引きはがし、押し潰し破壊、PUSH_ONLY、
   12 ブロック制限、`piston_head` / `moving_piston`、MovingBlock entity
8. **形状・サポート計算の共通ヘルパー** — 現行フック上で実装可能。trait 化して再利用
9. **ライトエンジン（emitLight）・サウンド / パーティクル** — 低優先

新しいファセット用の `CapSet` ビットは `8..32` の 24 個が空いており、`Fluid` / `Falling` /
`Explosive` / `Redstone` / `Growable` / `Oxidizable` / `Container` / `Usable` などを
ここに割り当てられる。

## 推奨ロードマップ

1. **P0**（既存フックのみ）: 流体 → 落下 → 形状・接続 → サポート → 火 → TNT
2. **P1**（既存フックのみ）: レッドストーン → 成長 → 酸化
3. **P2**（API 追加）: `on_use` 導入 → 開閉・操作系 → `ItemStack` 導入 → 収納系
4. **P3**（基盤追加）: エンティティ層 → 接触効果・輸送 → スポナー / ポータル / 機構

---

## 付録 A: 小カテゴリの全対象ブロック

### 流体（3）

`bubble_column`, `lava`, `water`

### 落下（26）

`anvil`, `black_concrete_powder`, `blue_concrete_powder`, `brown_concrete_powder`,
`chipped_anvil`, `cyan_concrete_powder`, `damaged_anvil`, `dragon_egg`, `gravel`,
`gray_concrete_powder`, `green_concrete_powder`, `light_blue_concrete_powder`,
`light_gray_concrete_powder`, `lime_concrete_powder`, `magenta_concrete_powder`,
`orange_concrete_powder`, `pink_concrete_powder`, `purple_concrete_powder`,
`red_concrete_powder`, `red_sand`, `sand`, `scaffolding`, `suspicious_gravel`,
`suspicious_sand`, `white_concrete_powder`, `yellow_concrete_powder`

### 爆発（2）

`respawn_anchor`, `tnt`

### 火（4）

`campfire`, `fire`, `soul_campfire`, `soul_fire`

### 輸送（4）

`activator_rail`, `detector_rail`, `powered_rail`, `rail`

## 付録 B: 形状ファミリの内訳

| ファミリ | 数 | 備考 |
| --- | ---: | --- |
| 階段 | 64 | `stair` 60 + 銅 4 |
| ハーフブロック | 68 | `slab` 64 + 銅 4 |
| 塀 | 32 | |
| フェンス | 13 | |
| ガラス窓・板ガラス | 約 18 | `stained_glass_pane` 16 + ガラス系 |
| 鉄格子 | 6 | |
| 柱（`axis`） | 56 | 原木・幹・深層岩など |
| ドア | 21 | `door` 17 + 銅 4 |
| トラップドア | 21 | `trapdoor` 17 + 銅 4 |
| フェンスゲート | 12 | |
| ボタン / 感圧板 / レバー | 31 | 14 + 16 + 1 |
| 看板・ハンギングサイン | 48 | 12 × 4 種 |
| バナー | 32 | 16 + 壁 16 |
| ベッド | 16 | |
| 頭部 | 13 | 頭・壁頭 各種 |

## 付録 C: 算出方法

- ブロック一覧と type 分類は `PrismarineJS/minecraft-data`（`pc_26_2`）の
  `blocks.json` と `misode/mcmeta`（`26.2-summary`）の `block_definitions` から生成。
- `examples/all_blocks.rs` の登録内容と同じデータに基づく。
- カテゴリの対象数は分類ルール（type と名前）による概数で、カテゴリ間の重複を含む。
  Piston の可動 / 不可動の割り当ては Minecraft Wiki の Piston/Table（Java Edition）に準拠。
