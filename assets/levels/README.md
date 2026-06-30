# 关卡 RON 写法

`assets/levels/*.ron` 描述单个关卡。当前默认关卡由
`src/game/level.rs` 中的 `DEFAULT_LEVEL_PATH` 指向 `level_01.ron`。
启动时可以用 `--level` / `-l` 指定其他关卡：

```powershell
cargo run -- --level assets/levels/level_row_three_physics_line.ron
```

## 顶层结构

```ron
(
    id: "level_01",
    display_name: "前院防线",
    starting_sun: 250,
    always_shoot: true,
    pea_path_arrival_effect: Straight,
    gatling_pea_upgrade_only: false,
    lawn: (
        columns: 9,
        cell_size: (90.0, 90.0),
        center_x: -50.0,
        path_y: -215.0,
    ),
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
- `always_shoot`: 是否让射手植物无论前方是否有僵尸都持续射击。省略时默认为 `false`。
- `pea_path_arrival_effect`: 底排豌豆到达 row 0 最左侧后的行为。省略时默认为 `Straight`。
- `gatling_pea_upgrade_only`: 是否让机枪射手只能替换已种下的多发射手，不能直接种在空格上。省略时默认为 `false`。
- `lawn.columns`: 底层草坪列数。
- `lawn.cell_size`: 单格宽高，单位是世界像素。
- `lawn.center_x`: 草坪整体中心 X。
- `lawn.path_y`: 僵尸行进道路中心 Y。
- `waves`: 显式大波列表。每个数组元素是一大波。

## pea_path_arrival_effect

该字段用于把不同物理实验隔离成不同关卡。当前可用值：

```ron
Straight
RowThreePhysicsLine
```

- `Straight`: 保留当前行为；底排豌豆向左到边界，再向上到 row 0，随后沿 row 0 向右飞行。
- `RowThreePhysicsLine`: 到达 row 0 最左侧后销毁原路径豌豆，并在 row 3 高度从草坪最左到最右生成 20 个物理豌豆。生成物理豌豆会继承触发弹丸当前的 `ProjectileKind` 与伤害，因此已经穿过火炬的火豌豆会变成物理火豌豆。

默认关卡 `level_01.ron` 使用 `Straight`。示例物理关卡
`level_row_three_physics_line.ron` 把后期波次压缩为总共 6 波，并把后一波的完整强度错峰叠加到前一波，终局波则叠加自身。所有波次从开场起保持高密度怪群与巨人队列，用于观察物理豌豆铺场效果；该关卡也开启了 `gatling_pea_upgrade_only`。

## 固定植物卡片

植物卡片不写在关卡 RON 中，由 `src/game/level.rs` 的固定列表定义：

```ron
Sunflower
TwinSunflower
Peashooter
Repeater
GatlingPea
SnowPea
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
