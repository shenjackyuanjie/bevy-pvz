# 关卡 RON 写法

`assets/levels/*.ron` 描述单个关卡。当前默认关卡由
`src/game/level.rs` 中的 `DEFAULT_LEVEL_PATH` 指向 `level_01.ron`。

## 顶层结构

```ron
(
    id: "level_01",
    display_name: "前院防线",
    starting_sun: 250,
    lawn: (
        columns: 9,
        cell_size: (90.0, 90.0),
        center_x: -50.0,
        path_y: -125.0,
    ),
    cards: [
        (slot: 1, plant: Sunflower),
        (slot: 2, plant: Peashooter),
    ],
    waves: [
        (
            delay: 8.0,
            wave: [
                (delay: 0.0, kind: Basic, count: 3, interval: 3.5),
                (delay: 6.0, kind: Conehead, count: 1, interval: 1.0),
            ],
        ),
    ],
)
```

## 字段说明

- `id`: 关卡 ID，不能为空。
- `display_name`: 显示名称，不能为空。
- `starting_sun`: 开局太阳数。
- `lawn.columns`: 底层草坪列数。
- `lawn.cell_size`: 单格宽高，单位是世界像素。
- `lawn.center_x`: 草坪整体中心 X。
- `lawn.path_y`: 僵尸行进道路中心 Y。
- `cards`: 可选植物卡片列表。`slot` 不能重复，`plant` 不能重复。
- `waves`: 显式大波列表。每个数组元素是一大波。

## cards

`plant` 使用 `PlantKind` 名称：

```ron
Sunflower
Peashooter
Repeater
GatlingPea
WallNut
Torchwood
```

## waves

每个大波结构如下：

```ron
(
    delay: 10.0,
    wave: [
        (delay: 0.0, kind: Basic, count: 5, interval: 2.5),
        (delay: 4.0, kind: Conehead, count: 2, interval: 1.0),
    ],
)
```

- 大波外层 `delay`: 距上一大波最后一只僵尸生成后的等待时间。第一大波表示开局等待。
- 内层 `wave`: 本大波里的刷怪条目，不能为空。
- 内层条目 `delay`: 相对于当前大波开始时间的延迟。
- `kind`: 僵尸种类，使用 `ZombieKind` 名称。
- `count`: 该条目生成数量，必须大于 0。
- `interval`: 同一条目内相邻僵尸的间隔。`count > 1` 时必须大于 0。

同一个大波内可以写多条相同或不同 `kind` 的刷怪条目。只要它们的内层
`delay` 相同，就会在同一时间生成，用于同时刷出多种僵尸。

解析时会把内层条目展开成绝对时间并排序，因此同一个大波内的条目顺序主要用于阅读。
下一大波的外层 `delay` 会从上一大波最后一个展开后的生成点之后开始计算。

## level_01.ron 用到的 ZombieKind

当前默认关卡使用了这些僵尸：

```ron
Basic
Conehead
PoleVaulting
Buckethead
Newspaper
ScreenDoor
Football
Zomboni
JackInTheBox
Balloon
Catapult
Digger
Pogo
Gargantuar
GigaGargantuar
Imp
```

完整可用列表见 `src/game/catalog.rs` 的 `ZombieKind`。
