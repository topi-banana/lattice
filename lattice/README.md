# lattice

`lattice` は、Minecraft 風のボクセルシミュレーションを **「ブロック = Rust の型」「能力 = ファセットトレイト」** として組み立てるための `no_std` ライブラリです。

ブロックの種類を固定テーブルで列挙するのではなく、ユーザーが型として定義し、能力（sticky / movable / instant minable …）をトレイトの合成として自由に与えられます。ワールドは 16³ の SubChunk を積んだ Chunk を `BTreeMap` で束ねる構造で、通常の `schedule` / `tick` ループを持ちます。

```rust
let palette = BlockPalette::from([
    Sticky + Movable + InstantMinable,
    Breakable + Movable,
    UnMovable,
    Empty,
]);
let mut world: World<'_, 2, 4> = World::new(&palette); // H=2 sections, P=4

world.set_block(&palette[0], (1, 1, 1));
world.set_block(&palette[2], (2, 1, 1));
world.schedule((1, 1, 1));
world.tick();
```

## 特徴

- **ブロックは型**: `BlockDef` を実装した `'static` な型がブロック定義。セルはパレット番号と生 `u16` ステートしか持たない。
- **能力は2つの顔**: `Sticky` はトレイト（`T: Sticky + Movable` の境界）としても、`CapSet` の値（`Sticky + Movable` の合成）としても使える。型名前空間と値名前空間の分離を利用している。
- **定義の同一性**: 同じ `T` は `eq`。能力が同じでも別の型 `S` なら `ne`。同じブロックでも境界（`movable()` と `sticky()`）が違えば `Facet` として `ne`。
- **動的チャンク**: `World<H, P>` は `BTreeMap<ChunkPos, Chunk<H>>` を持ち、水平方向は無限・負の座標も可。書き込み時にチャンクを自動確保し、`load_chunk` / `unload_chunk` で明示的に扱うこともできる。
- **決定的な tick**: SubChunk ごとの `[u64; 64]` スケジュールビットセット。tick 中に再スケジュールした分は次回に回り、tick 中にロードされたチャンクは次回に走査される。
- **フックから世界を操作**: `TickCtx` 経由で読み書き・スケジュール・破壊・削除ができる。設置時には `on_placed` が走る。
- **Block entity（セル外の状態）**: 16bit に収まらない状態は `BlockEntity` として、セルとは別のチャンク内 `BTreeMap<Pos, Box<dyn BlockEntity>>` に保持する。block state と block entity は独立したストアなので、空気セルだけに entity が存在する「孤立 entity」も表現でき、`schedule` された位置はブロック → entity の順に両方のフックが走る。
- **同梱の機構**: `Power` 能力と `PistonBlock` / `PowerBlock`。動力を受けたピストンは 6gt 後に前方の列を押し出す。
- **`#![no_std]`**: 依存は `core` + `alloc`（チャンクの `BTreeMap`・block entity マップ・tick の作業バッファのみ）。OS 依存なし、`unsafe` なし。

## 実行

```console
$ cargo run --example basic             # 最小のパレット + schedule + tick
$ cargo run --example custom_blocks     # 独自ブロック定義と StateCodec
$ cargo run --example piston            # ピストン + レッドストーン: 設置 → 6gt → 移動
$ cargo run --example entities          # block entity: セル外の状態と孤立 entity の tick
$ cargo test                            # 単体テスト + ドキュメントテスト
```

## ストレージ構造

```text
World<'a, H, P>
  palette:  &BlockPalette<P>
  chunks:   BTreeMap<ChunkPos, Chunk<H>>   書き込み時に自動確保
  scratch:  Vec<ChunkPos>                  tick 中のチャンク走査用（再利用）

Chunk<H>
  subchunks: [SubChunk; H]
  entities:  BTreeMap<Pos, Box<dyn BlockEntity>>   セルとは独立した block entity マップ

SubChunk
  cells:     [[[Cell; 16]; 16]; 16]   16³ = 4096 セル（index = (x*16 + y)*16 + z）
  scheduled: [u64; 64]                4096 ビットのスケジュールビットセット
  scheduled_count: u32

Cell
  block: u16                          パレット index
  state: u16                          ブロックごとの生ステート
```

- `ChunkPos { x, z }` は水平方向のチャンク列。ブロック座標は床除算で対応するため、負の座標も自然に扱える（`x = -1` は `ChunkPos::new(-1, ..)`）。
- 垂直方向は `H` 個の SubChunk 固定（`H = 4` なら 64 ブロック）。`y` が `0..H*16` の外に出ると `get` は `None`、`set_block` は拒否される。
- チャンクは最初の書き込みで自動確保される。読み込み前の `get` は `None`、`schedule` は `false` を返す。
- tick は「ロード済みチャンクの座標をスナップショット → BTreeMap 座標順 → SubChunk 順 → ビット順」で決定的。フックがチャンクをロード/アンロードしても安全で、新規チャンクは次回の tick で走査される。

## 設計

### 1. 能力をトレイトと値の二重表現にする

```rust
use lattice::{BlockDef, CapSet, Movable, Sticky};

block! {
    /// Slime のようなブロック
    pub struct Slime;
    facets: Sticky, Movable;
}
```

`block!` マクロは `impl Sticky for Slime` / `impl Movable for Slime` を生成し、同時に

```rust
const CAPS: CapSet = CapSet::STICKY.or(CapSet::MOVABLE);
```

を各ファセットの `CAP` から自動計算します。つまりコンパイル時に見える境界と実行時に見える能力がひとつのソースから生成されます。

手書きでも定義できます（この場合は `CAPS` を自分で整合させます）:

```rust
use lattice::{BlockDef, CapSet};

struct Slime;

impl BlockDef for Slime {
    const CAPS: CapSet = CapSet::STICKY.or(CapSet::MOVABLE);
    type State = ();
}
```

`CapSet` のビット `8..32` はユーザー用に空けてあり、独自能力を追加できます:

```rust
pub const EXPLOSIVE: CapSet = CapSet::bit(CapSet::USER_BIT_START);

pub trait Explosive: BlockDef {
    const CAP: CapSet = CapSet::bit(CapSet::USER_BIT_START);
}

block! {
    pub struct Tnt;
    facets: Explosive, InstantMinable;
}
```

`BlockPalette::from([...])` は同じ型の配列しか取れないため、型定義とインライン能力を混ぜるときは `palette!` マクロを使います:

```rust
let palette = palette![Slime::source(), Sticky + Movable, Empty];
```

### 2. 同一性・等価性・境界

| 比較 | 結果 | 理由 |
| --- | --- | --- |
| `palette[0] == palette[0]` | `eq` | 同じ定義（`Identity`） |
| `T` vs `S`（能力が同じ） | `ne` | 名前的 identity は `TypeId` |
| `Sticky + Movable` vs `Movable + Sticky` | `eq` | 構造的 identity は能力集合 |
| `block.movable() == block.movable()` | `eq` | 同じ定義・同じ境界 |
| `block.movable() == block.sticky()` | `ne` | 同じ定義・違う境界 |
| `t.movable() == s.movable()` | `ne` | 違う定義・同じ境界 |

`Facet` は「ブロックをひとつの能力境界から見たもの」で、`(定義, 境界)` のペアとして比較されます。ユーザー定義型は `TypeId` で、インライン定義は能力集合で同一性を持つため、`T: Movable` と `S: Movable` が能力を共有していても等価にはなりません。

パレット構築時には重複が正規化されます。同じ能力集合の構造的定義、同じ `TypeId` の定義は 1 エントリにまとめられ、異なるパレットから取り出した同じ定義の `Block` も `eq` になります。

### 3. ブロック操作

- `set_block` / `set_block_with_state` はチャンクを自動ロードし、`on_neighbor_changed` を隣接 6 セルに通知する。
- `remove_block` は能力を無視して空気に置き換える。
- `break_block(pos)` は `InstantMinable` なら即時削除 + `on_break`、`Breakable` なら残り tick 数を返す。
- 列を動かすのはワールドの API ではなく `PistonBlock` 自身の仕事。ピストンは `ctx.get` で前方を走査し、`Movable`（かつ `UnMovable` でない）ブロックだけを 1 マス動かす。移動は `ctx.move_cell` 相当の内部プリミティブで生ステートごと行われ、最後に隣接へ通知される。
- `positions_of::<T>()` / `positions(caps)` はロード済みチャンクを座標順に走査するイテレータを返す。
- `random_tick(&mut Rng32, count)` はロード済みチャンクから一様にセルを選び、`on_random_tick` を呼ぶ。

フックはすべて関連関数で、セルごとの可変状態は `TickCtx` 経由で読み書きします:

```rust
impl BlockDef for Fuse {
    const CAPS: CapSet = CapSet::BREAKABLE;
    type State = u8; // 残り tick

    fn on_scheduled_tick(ctx: &mut TickCtx<'_>) {
        let remaining = ctx.state::<Self>();
        if remaining == 0 {
            ctx.remove_self();
        } else {
            ctx.set_state::<Self>(remaining - 1);
            ctx.schedule_self();
        }
    }
}
```

### 4. Block entity（セル外の状態）

セルはパレット番号と生 `u16` しか持てません。インベントリや 32bit カウンタなど、それに収まらない状態は `BlockEntity` として位置に紐づけます。block entity はセル配列とは別の、チャンク内 `BTreeMap<Pos, Box<dyn BlockEntity>>` に保持され、ブロックと entity は独立に読み書きできます。

```rust
use lattice::{BlockEntity, EntityTickCtx};

#[derive(Default)]
pub struct Counter {
    pub ticks: u32,
}

impl BlockEntity for Counter {
    fn on_scheduled_tick(&mut self, ctx: &mut EntityTickCtx<'_>) {
        self.ticks += 1;
        ctx.schedule_self();
    }
}

block! {
    pub struct Ticker;
    facets: UnMovable;
    entity: Counter; // 設置時に Counter::default() が生成される
}
```

| 操作 | block entity への効果 |
| --- | --- |
| `set_block`（定義が変わる） | 旧 entity を破棄し、新定義の `create_block_entity` ファクトリを実行 |
| `set_block` / `set_block_with_state`（同一定義） | 変更なし（state のみ更新） |
| `remove_block` / `break_block` | ブロックと一緒に削除。`on_break` からはまだ見える |
| ピストンの移動 | セルと一緒に移動。移動先に entity がある場合はストローク全体が失敗 |
| `set_block_entity` / `remove_block_entity` | 手動。ブロックに触れず、空気セルへの「孤立 entity」も作れる |

- 実行は schedule ベース。`world.tick()` はスケジュールされた位置ごとに、ブロックの `on_scheduled_tick` → entity の `on_scheduled_tick` の順に呼ぶ（`random_tick` も同様に `on_random_tick` を順に呼ぶ）。孤立 entity だけの位置でも entity は tick する。
- entity のフックは `&mut self` を受け取るため、実行中は entity が world から一時的に切り離される（自分の位置を `ctx.block_entity` で引くと `None`）。フックが戻ると、`remove_self` が呼ばれず・置き換えもなく・ホストブロックの定義が変わっていなければ再び接続される。
- `World::block_entity` / `entity::<T>` はブロックとは独立に引ける。ファクトリを持たない定義に手動で付ける、entity だけ消してブロックを残す、といった不一致もそのまま表現できる。

### 5. ピストン機構（PistonBlock / PowerBlock）

機構の実例として `PistonBlock` と `PowerBlock` が同梱されています。`PowerBlock` は `Power` 能力を持ち、`PistonBlock` は設置時 (`on_placed`) と隣接更新時 (`on_neighbor_changed`) に隣接 6 セルの `Power` 能力を確認します。すべてブロックのフックから `TickCtx` 経由で駆動され、`world.tick()` だけが時計です。

```rust
let palette = palette![PistonBlock::source(), PowerBlock::source(), Movable, Empty];
let mut world: World<'_, 1, 4> = World::new(&palette);

let piston = palette.find_type::<PistonBlock>().unwrap();
let power = palette.find_type::<PowerBlock>().unwrap();
let movable = palette[2];

// 向きを指定して設置（状態 = phase / facing / timer）
world.set_block_with_state(piston, (0, 0, 0), PistonState::retracted(Facing::PosX).encode());
world.set_block(movable, (1, 0, 0));

// 動力の設置が update としてピストンに届き、6gt のカウントダウンが始まる
world.set_block(power, (-1, 0, 0));

// 6gt 後に前方のブロックが 1 マス動く
for _ in 0..6 {
    world.tick();
}
assert_eq!(world.get(Pos::new(2, 0, 0)).unwrap(), movable);
```

タイミングはピストンのセル状態で管理します:

| tick | phase | timer | 動作 |
| --- | --- | --- | --- |
| 設置 / update | `Extending` | 6 | `schedule_self()` |
| 1〜5 | `Extending` | 5..1 | 減算して再スケジュール |
| 6 | `Extended` | 0 | 前方の可動ブロック列を 1 マス移動（生ステートごと） |

- 動力を失うと `Extending` / `Extended` のどちらからでも `Retracted` に戻り、再給電でまた 6gt 待ちます。
- 押せないブロック（`UnMovable` や 16 マス超）に当たると押し出しは原子的に失敗し、ピストンは `Retracted` のまま待機します。
- 移動するセルに付いた block entity はセルと一緒に運ばれます。移動先に既に entity がある場合（孤立 entity を含む）はストロークが失敗します。
- 移動アニメーション用の MovingBlock entity はまだ無いため、移動は 6gt 目の 1 ステップで完結します。instant move や収縮時の引きはがし（sticky piston）は未実装です。

### 6. no_std ポリシー

本体は `#![no_std]`、`core` + `alloc` のみに依存します。`alloc` を使うのはチャンクの `BTreeMap`・チャンクごとの block entity マップ・tick の作業バッファだけで、パレット・定義・セル・スケジュールビットセットはアロケーションしません（entity も実際に付けた分だけアロケートします）。OS 依存も `unsafe` もありません。`alloc` の供給は最終バイナリ側の役割です（examples / tests は std が供給）。

```console
$ cargo check --target x86_64-unknown-none
```

## ディレクトリ

```
src/
  caps.rs       CapSet とファセットトレイト
  def.rs        BlockDef / Identity / StateCodec
  entity.rs     BlockEntity / EntityTickCtx / AsAny
  palette.rs    BlockPalette / Block / BlockSource / Facet
  chunk.rs      SubChunk / Chunk<H> / ChunkPos
  piston.rs     PistonBlock / PowerBlock / PistonState / Facing
  world.rs      World<H, P> / TickCtx / schedule / break
  pos.rs        Pos / Dir
  rng.rs        Rng32
examples/
  basic.rs            最小例
  custom_blocks.rs    独自定義 + ステート + `T: Sticky + Movable` 境界
  piston.rs           ピストン + レッドストーン: 設置 → 6gt → 移動
  entities.rs         block entity + 孤立 entity の tick
tests/
  definitions.rs  同一性・境界・能力ビット
  simulation.rs   tick・隣接・破壊・押し出し・チャンクのライフサイクル
  piston.rs       ピストン機構（6gt・動力・向き・ブロック・リセット）
  entities.rs     block entity のライフサイクル・孤立・tick・ピストン搬送
```

## 今後の拡張候補

- MovingBlock entity による移動アニメーションと、収縮時の引きはがし（sticky piston）
- instant move（0gt）や押し出し数・挙動のピストンごとのカスタム
- リージョンファイルやシリアライズ（チャンク単位の保存・復元）
- 垂直方向の可変化（SubChunk の上下追加）やセクション単位のロード
- 流体・マルチブロック構造など、より大きなシミュレーション語彙
- イベント購読（破壊・移動・隣接更新）と観測フック
- 能力集合に対する型レベル述語（`T::Facets: Contains<Sticky>`）による境界の自動導出
