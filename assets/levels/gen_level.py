# Generate the RON level file with calculated timing
# Target: last zombie spawn at 365s (6m5s)
# Usage: python gen_level.py

TARGET_LAST_SPAWN_SECONDS = 6 * 60 + 5

# Output six waves, then promote the full intensity of each following wave into
# the current wave. The final wave doubles its own intensity as the next tier.
WAVE_GROUPS = [
    (0, 1, "第 1 波：强压开场 — 叠加第 2 波完整强度"),
    (1, 6, "第 2 波：巨人入场 — 叠加第 3 波完整强度"),
    (6, 11, "第 3 波：全面战争 — 叠加第 4 波完整强度"),
    (11, 16, "第 4 波：绝望之潮 — 叠加第 5 波完整强度"),
    (16, 21, "第 5 波：终局前夜 — 叠加第 6 波完整强度"),
    (21, 25, "第 6 波：终局翻倍 — 再叠加一轮终局强度"),
]

# Extra output-wave fillers for waves 3-6. Each entry is:
# (start_fraction, span_fraction, kind, count).
# They add staggered spawn frequencies without extending the target timeline.
LATE_WAVE_FILLERS = {
    2: [
        (0.02, 0.92, "Basic", 40),
        (0.08, 0.86, "Conehead", 32),
        (0.14, 0.80, "Buckethead", 24),
        (0.21, 0.72, "Football", 18),
        (0.05, 0.88, "Gargantuar", 24),
        (0.17, 0.72, "GigaGargantuar", 12),
    ],
    3: [
        (0.02, 0.92, "Basic", 42),
        (0.08, 0.86, "Conehead", 36),
        (0.14, 0.80, "Imp", 48),
        (0.22, 0.72, "Buckethead", 30),
        (0.30, 0.64, "Football", 20),
        (0.04, 0.90, "Gargantuar", 36),
        (0.16, 0.76, "GigaGargantuar", 18),
    ],
    4: [
        (0.02, 0.92, "Conehead", 50),
        (0.07, 0.88, "Imp", 60),
        (0.13, 0.82, "Buckethead", 38),
        (0.21, 0.74, "Football", 28),
        (0.31, 0.62, "ScreenDoor", 20),
        (0.03, 0.91, "Gargantuar", 48),
        (0.14, 0.79, "GigaGargantuar", 24),
    ],
    5: [
        (0.01, 0.94, "Imp", 70),
        (0.06, 0.90, "Conehead", 60),
        (0.12, 0.84, "Buckethead", 48),
        (0.19, 0.78, "Football", 36),
        (0.28, 0.68, "Pogo", 22),
        (0.37, 0.58, "Digger", 18),
        (0.02, 0.94, "Gargantuar", 60),
        (0.12, 0.84, "GigaGargantuar", 36),
    ],
}

BASE_WAVES_CONFIG = [
    # (wave_delay, entries[(entry_delay, kind, count, interval)])

    # === WAVE 1: Warmup ===
    (8.0, [
        (0, "Basic", 15, 4.0),
        (3, "Conehead", 8, 6.0),
        (10, "PoleVaulting", 4, 8.0),
    ]),

    # === WAVE 2: Pressure ===
    (10.0, [
        (0, "Basic", 12, 4.0),
        (3, "Conehead", 10, 5.0),
        (8, "Buckethead", 6, 6.0),
        (14, "Newspaper", 5, 7.0),
    ]),

    # === WAVE 3: 2 Gargantuar debut ===
    (10.0, [
        (0, "Basic", 10, 4.0),
        (3, "Gargantuar", 2, 16.0),
        (4, "Conehead", 8, 5.0),
        (10, "ScreenDoor", 5, 7.0),
        (14, "Football", 5, 6.0),
    ]),

    # === WAVE 4: 4 Gargantuar ===
    (10.0, [
        (0, "Conehead", 12, 3.0),
        (3, "Gargantuar", 2, 14.0),
        (4, "Buckethead", 8, 4.0),
        (7, "Gargantuar", 2, 14.0),
        (10, "PoleVaulting", 6, 6.0),
        (18, "Zomboni", 2, 14.0),
    ]),

    # === WAVE 5: 6 Gargantuar ===
    (10.0, [
        (0, "Basic", 12, 3.0),
        (3, "Gargantuar", 3, 13.0),
        (4, "Buckethead", 10, 4.0),
        (7, "Gargantuar", 3, 13.0),
        (10, "Football", 6, 5.0),
        (14, "ScreenDoor", 6, 6.0),
        (22, "Imp", 6, 4.0),
    ]),

    # === WAVE 6: 8 Gargantuar ===
    (10.0, [
        (0, "Conehead", 10, 3.0),
        (3, "Gargantuar", 4, 12.0),
        (4, "Buckethead", 10, 4.0),
        (7, "Gargantuar", 4, 12.0),
        (10, "JackInTheBox", 4, 8.0),
        (12, "Football", 8, 5.0),
        (20, "Imp", 8, 4.0),
    ]),

    # === WAVE 7: 10 Gargantuar ===
    (10.0, [
        (0, "Gargantuar", 5, 11.0),
        (2, "Conehead", 10, 3.0),
        (5, "Gargantuar", 5, 11.0),
        (8, "Balloon", 4, 8.0),
        (12, "Catapult", 4, 9.0),
        (15, "Football", 8, 4.0),
        (20, "Imp", 10, 3.0),
    ]),

    # === WAVE 8: 12 Gargantuar ===
    (10.0, [
        (0, "Gargantuar", 6, 10.0),
        (2, "Buckethead", 10, 3.0),
        (5, "Gargantuar", 6, 10.0),
        (8, "Digger", 4, 8.0),
        (12, "Pogo", 4, 9.0),
        (15, "Football", 10, 4.0),
        (22, "Imp", 10, 3.0),
    ]),

    # === WAVE 9: 14 (introducing GigaGargantuar) ===
    (10.0, [
        (0, "Gargantuar", 5, 10.0),
        (3, "GigaGargantuar", 2, 18.0),
        (5, "Gargantuar", 5, 10.0),
        (7, "GigaGargantuar", 2, 18.0),
        (10, "Catapult", 5, 8.0),
        (14, "Balloon", 4, 9.0),
        (18, "Football", 10, 4.0),
        (25, "Imp", 12, 3.0),
    ]),

    # === WAVE 10: 16 total giants ===
    (10.0, [
        (0, "Gargantuar", 4, 10.0),
        (3, "GigaGargantuar", 4, 16.0),
        (5, "Gargantuar", 4, 10.0),
        (7, "GigaGargantuar", 4, 16.0),
        (10, "Zomboni", 4, 10.0),
        (14, "Pogo", 6, 7.0),
        (18, "Football", 8, 4.0),
        (24, "Imp", 12, 3.0),
    ]),

    # === WAVE 11: 10 Garg + 8 Giga = 18 ===
    (10.0, [
        (0, "Gargantuar", 5, 9.0),
        (3, "GigaGargantuar", 4, 15.0),
        (5, "Gargantuar", 5, 9.0),
        (7, "GigaGargantuar", 4, 15.0),
        (10, "Digger", 5, 7.0),
        (14, "Balloon", 5, 8.0),
        (18, "Football", 10, 4.0),
        (25, "Imp", 14, 3.0),
    ]),

    # === WAVE 12: 20 total giants ===
    (10.0, [
        (0, "Gargantuar", 5, 9.0),
        (3, "GigaGargantuar", 5, 14.0),
        (5, "Gargantuar", 5, 9.0),
        (7, "GigaGargantuar", 5, 14.0),
        (10, "Pogo", 6, 7.0),
        (14, "Zomboni", 5, 9.0),
        (18, "Football", 10, 4.0),
        (25, "Catapult", 5, 8.0),
        (30, "Imp", 14, 3.0),
    ]),

    # === WAVE 13: 12 Garg + 10 Giga = 22 ===
    (10.0, [
        (0, "Gargantuar", 6, 9.0),
        (3, "GigaGargantuar", 5, 14.0),
        (5, "Gargantuar", 6, 9.0),
        (7, "GigaGargantuar", 5, 14.0),
        (10, "Balloon", 6, 7.0),
        (14, "Pogo", 6, 7.0),
        (18, "Football", 12, 4.0),
        (25, "Imp", 16, 3.0),
    ]),

    # === WAVE 14: Sustained ===
    (10.0, [
        (0, "Gargantuar", 6, 8.0),
        (3, "GigaGargantuar", 5, 13.0),
        (5, "Gargantuar", 6, 8.0),
        (7, "GigaGargantuar", 5, 13.0),
        (10, "Digger", 6, 7.0),
        (14, "Catapult", 6, 8.0),
        (18, "Football", 12, 4.0),
        (22, "Zomboni", 5, 9.0),
        (28, "Imp", 16, 3.0),
    ]),

    # === WAVE 15 ===
    (10.0, [
        (0, "Gargantuar", 6, 8.0),
        (3, "GigaGargantuar", 6, 13.0),
        (5, "Gargantuar", 6, 8.0),
        (7, "GigaGargantuar", 6, 13.0),
        (10, "Pogo", 7, 6.0),
        (14, "Balloon", 6, 7.0),
        (18, "Football", 12, 4.0),
        (22, "Digger", 6, 7.0),
        (30, "Imp", 18, 3.0),
    ]),

    # === WAVE 16: Longer tails ===
    (10.0, [
        (0, "Gargantuar", 6, 8.0),
        (3, "GigaGargantuar", 6, 12.0),
        (5, "Gargantuar", 6, 8.0),
        (7, "GigaGargantuar", 6, 12.0),
        (10, "Catapult", 6, 7.0),
        (14, "Zomboni", 6, 8.0),
        (18, "Football", 12, 4.0),
        (22, "Pogo", 6, 7.0),
        (28, "Balloon", 6, 7.0),
        (35, "Imp", 18, 3.0),
    ]),

    # === WAVE 17 ===
    (10.0, [
        (0, "Gargantuar", 6, 7.0),
        (3, "GigaGargantuar", 6, 12.0),
        (5, "Gargantuar", 6, 7.0),
        (7, "GigaGargantuar", 6, 12.0),
        (10, "Digger", 6, 6.0),
        (14, "Balloon", 6, 7.0),
        (18, "Football", 14, 4.0),
        (22, "Catapult", 6, 7.0),
        (28, "Pogo", 6, 7.0),
        (35, "Imp", 20, 3.0),
    ]),

    # === WAVE 18 ===
    (10.0, [
        (0, "Gargantuar", 6, 7.0),
        (3, "GigaGargantuar", 7, 12.0),
        (5, "Gargantuar", 6, 7.0),
        (7, "GigaGargantuar", 7, 12.0),
        (10, "Zomboni", 6, 7.0),
        (14, "Pogo", 6, 6.0),
        (18, "Football", 14, 4.0),
        (22, "Digger", 6, 7.0),
        (28, "Balloon", 6, 7.0),
        (36, "Imp", 20, 3.0),
    ]),

    # === WAVE 19 ===
    (10.0, [
        (0, "Gargantuar", 6, 7.0),
        (3, "GigaGargantuar", 7, 11.0),
        (5, "Gargantuar", 6, 7.0),
        (7, "GigaGargantuar", 7, 11.0),
        (10, "Catapult", 6, 6.0),
        (14, "Balloon", 6, 7.0),
        (18, "Football", 14, 4.0),
        (22, "Zomboni", 6, 7.0),
        (28, "Pogo", 6, 7.0),
        (36, "Imp", 20, 3.0),
    ]),

    # === WAVE 20 ===
    (10.0, [
        (0, "Gargantuar", 6, 7.0),
        (3, "GigaGargantuar", 8, 11.0),
        (5, "Gargantuar", 6, 7.0),
        (7, "GigaGargantuar", 8, 11.0),
        (10, "Digger", 6, 6.0),
        (14, "Pogo", 6, 6.0),
        (18, "Football", 14, 4.0),
        (22, "Catapult", 6, 7.0),
        (28, "Balloon", 6, 7.0),
        (35, "Zomboni", 6, 7.0),
        (42, "Imp", 20, 3.0),
    ]),

    # === WAVE 21 ===
    (10.0, [
        (0, "Gargantuar", 6, 7.0),
        (3, "GigaGargantuar", 8, 11.0),
        (5, "Gargantuar", 6, 7.0),
        (7, "GigaGargantuar", 8, 11.0),
        (10, "Balloon", 6, 6.0),
        (14, "Catapult", 6, 6.0),
        (18, "Football", 16, 4.0),
        (22, "Digger", 6, 7.0),
        (28, "Pogo", 6, 7.0),
        (35, "Zomboni", 6, 7.0),
        (42, "Imp", 20, 3.0),
        (48, "Gargantuar", 4, 8.0),
    ]),

    # === WAVE 22 ===
    (10.0, [
        (0, "Gargantuar", 6, 7.0),
        (3, "GigaGargantuar", 8, 11.0),
        (5, "Gargantuar", 6, 7.0),
        (7, "GigaGargantuar", 8, 11.0),
        (10, "Pogo", 6, 6.0),
        (14, "Zomboni", 6, 7.0),
        (18, "Football", 16, 4.0),
        (22, "Balloon", 6, 6.0),
        (28, "Catapult", 6, 7.0),
        (35, "Digger", 6, 7.0),
        (42, "Imp", 22, 3.0),
        (48, "Gargantuar", 4, 8.0),
    ]),

    # === WAVE 23 ===
    (10.0, [
        (0, "Gargantuar", 6, 7.0),
        (3, "GigaGargantuar", 8, 11.0),
        (5, "Gargantuar", 6, 7.0),
        (7, "GigaGargantuar", 8, 11.0),
        (10, "Catapult", 6, 6.0),
        (14, "Balloon", 6, 6.0),
        (18, "Football", 16, 4.0),
        (22, "Pogo", 6, 6.0),
        (28, "Digger", 6, 7.0),
        (35, "Zomboni", 6, 7.0),
        (42, "Imp", 28, 3.0),
        (48, "Gargantuar", 6, 8.0),
        (55, "GigaGargantuar", 4, 10.0),
    ]),

    # === WAVE 24: Extended war ===
    (10.0, [
        (0, "Gargantuar", 6, 7.0),
        (3, "GigaGargantuar", 8, 11.0),
        (5, "Gargantuar", 6, 7.0),
        (7, "GigaGargantuar", 8, 11.0),
        (10, "Pogo", 6, 6.0),
        (14, "Zomboni", 6, 7.0),
        (18, "Football", 16, 4.0),
        (22, "Balloon", 6, 6.0),
        (28, "Catapult", 6, 7.0),
        (35, "Digger", 6, 7.0),
        (42, "Imp", 28, 3.0),
        (48, "Gargantuar", 6, 8.0),
        (55, "GigaGargantuar", 4, 10.0),
    ]),

    # === WAVE 25: Final stand ===
    (10.0, [
        (0, "Gargantuar", 6, 7.0),
        (3, "GigaGargantuar", 8, 11.0),
        (5, "Gargantuar", 6, 7.0),
        (7, "GigaGargantuar", 8, 11.0),
        (10, "Pogo", 6, 6.0),
        (14, "Zomboni", 6, 7.0),
        (18, "Football", 18, 4.0),
        (22, "Balloon", 6, 6.0),
        (28, "Catapult", 6, 7.0),
        (35, "Digger", 6, 7.0),
        (42, "Imp", 26, 3.0),
        (50, "Gargantuar", 6, 8.0),
        (58, "GigaGargantuar", 6, 10.0),
    ]),
]


def wave_duration(entries):
    return max(
        entry_delay + (count - 1) * interval
        for entry_delay, _kind, count, interval in entries
    )


def timeline_total_seconds(waves):
    elapsed = 0.0
    for delay, entries in waves:
        wave_start = elapsed + delay
        elapsed = wave_start + wave_duration(entries)
    return elapsed


def total_spawn_count(waves):
    return sum(count for _delay, entries in waves for _entry_delay, _kind, count, _interval in entries)


def base_wave_timeline(waves):
    timeline = []
    elapsed = 0.0
    for delay, entries in waves:
        start = elapsed + delay
        last = start + wave_duration(entries)
        timeline.append((start, last, entries))
        elapsed = last
    return timeline


def grouped_scaled_waves_config(base_waves, wave_groups):
    timeline = base_wave_timeline(base_waves)
    if wave_groups[0][:2] != (0, 1):
        raise ValueError("the first output wave is expected to contain only base wave 1")

    first_wave_end = timeline[0][1]
    base_total = timeline[-1][1]
    scale = (TARGET_LAST_SPAWN_SECONDS - first_wave_end) / (base_total - first_wave_end)
    if scale <= 0:
        raise ValueError("target is too small for the configured first wave")

    grouped = []
    previous_last = 0.0
    for group_index, (start_index, end_index, _comment) in enumerate(wave_groups):
        if not (0 <= start_index < end_index <= len(timeline)):
            raise ValueError("invalid wave group range")

        if group_index == 0:
            group_start = timeline[start_index][0]
            entries = [
                (entry_delay, kind, count, interval)
                for entry_delay, kind, count, interval in timeline[start_index][2]
            ]
        else:
            original_group_start = timeline[start_index][0]
            group_start = first_wave_end + (original_group_start - first_wave_end) * scale
            entries = []
            for wave_start, _wave_last, wave_entries in timeline[start_index:end_index]:
                for entry_delay, kind, count, interval in wave_entries:
                    original_entry_start = wave_start + entry_delay
                    entry_start = first_wave_end + (original_entry_start - first_wave_end) * scale
                    entries.append((entry_start - group_start, kind, count, interval * scale))

        wave_delay = group_start - previous_last
        grouped.append((wave_delay, entries))
        previous_last = group_start + wave_duration(entries)

    return grouped, scale


def add_late_wave_fillers(waves, filler_config):
    filled = []
    for wave_index, (delay, entries) in enumerate(waves):
        entries = list(entries)
        duration = wave_duration(entries)
        for start_fraction, span_fraction, kind, count in filler_config.get(wave_index, []):
            if not (0.0 <= start_fraction <= 1.0):
                raise ValueError("filler start fraction must be in [0, 1]")
            if span_fraction < 0.0 or start_fraction + span_fraction > 1.0:
                raise ValueError("filler span must stay within the wave duration")
            if count <= 0:
                raise ValueError("filler count must be positive")

            entry_delay = duration * start_fraction
            interval = 0.0 if count == 1 else duration * span_fraction / (count - 1)
            entries.append((entry_delay, kind, count, interval))
        entries.sort(key=lambda entry: (entry[0], entry[1]))
        filled.append((delay, entries))
    return filled


def promote_next_wave_intensity(waves):
    promoted = []
    for wave_index, (delay, entries) in enumerate(waves):
        target_duration = wave_duration(entries)
        source_entries = waves[min(wave_index + 1, len(waves) - 1)][1]
        source_duration = wave_duration(source_entries)

        # Keep the promoted pattern inside the original wave boundary so the
        # overall 365-second timeline remains unchanged and spawn points differ.
        time_scale = target_duration * 0.94 / source_duration
        time_offset = target_duration * 0.03
        promoted_entries = list(entries)
        promoted_entries.extend(
            (
                time_offset + entry_delay * time_scale,
                kind,
                count,
                interval * time_scale,
            )
            for entry_delay, kind, count, interval in source_entries
        )
        promoted_entries.sort(key=lambda entry: (entry[0], entry[1]))
        promoted.append((delay, promoted_entries))
    return promoted


waves_config, time_scale = grouped_scaled_waves_config(BASE_WAVES_CONFIG, WAVE_GROUPS)
waves_config = add_late_wave_fillers(waves_config, LATE_WAVE_FILLERS)
waves_config = promote_next_wave_intensity(waves_config)

# =============================================
# Timing Analysis
# =============================================
print("=== Timing Analysis ===", flush=True)
print(
    f"Base correct total: {timeline_total_seconds(BASE_WAVES_CONFIG):.1f}s",
    flush=True,
)
print(f"Output waves: {len(waves_config)}", flush=True)
print(f"Uncompressed first wave end: {base_wave_timeline(BASE_WAVES_CONFIG)[0][1]:.1f}s", flush=True)
print(f"Compressed time scale after first wave: {time_scale:.6f}", flush=True)
print(f"Total zombies: {total_spawn_count(waves_config)}", flush=True)

elapsed = 0.0
all_last_spawns = []
for i, (delay, entries) in enumerate(waves_config, start=1):
    start = elapsed + delay
    entry_ends = []
    for edelay, kind, count, interval in entries:
        end = start + edelay + (count - 1) * interval
        entry_ends.append((kind, count, end))
    last = max(end for _, _, end in entry_ends)
    all_last_spawns.append(last)
    elapsed = last

    garg_count = sum(c for _, k, c, _ in entries if k in ("Gargantuar", "GigaGargantuar"))
    print(f"Wave {i:2d}: start={start:6.1f}, last_spawn={last:6.1f}s, giants={garg_count:2d}", flush=True)

total = max(all_last_spawns)
print(f"\nOverall last spawn: {total:.1f}s = {int(total//60)}m{int(total%60)}s", flush=True)
print(f"Target: {TARGET_LAST_SPAWN_SECONDS}s = 6m5s", flush=True)
print(f"Difference: {total - TARGET_LAST_SPAWN_SECONDS:.3f}s", flush=True)

# =============================================
# RON Generation
# =============================================
INDENT = " " * 4
INDENT2 = " " * 8
INDENT3 = " " * 12

def fmt_num(v):
    """Format number with stable precision and .0 suffix for whole floats."""
    rounded = round(v, 6)
    if abs(rounded - round(rounded)) < 0.000001:
        return f"{int(round(rounded))}.0"
    text = f"{rounded:.6f}".rstrip("0").rstrip(".")
    if "." not in text:
        text += ".0"
    return text

WAVE_COMMENTS = [comment for _start, _end, comment in WAVE_GROUPS]

lines = []
lines.append("(")
lines.append(f'{INDENT}id: "level_row_three_physics_line",')
lines.append(f'{INDENT}display_name: "物理豌豆横列·地狱",')
lines.append(f"{INDENT}starting_sun: 200,")
lines.append(f"{INDENT}always_shoot: true,")
lines.append(f"{INDENT}pea_path_arrival_effect: RowThreePhysicsLine,")
lines.append(f"{INDENT}gatling_pea_upgrade_only: true,")
lines.append(f"{INDENT}lawn: (")
lines.append(f"{INDENT2}columns: 9,")
lines.append(f"{INDENT2}cell_size: (90.0, 90.0),")
lines.append(f"{INDENT2}center_x: -50.0,")
lines.append(f"{INDENT2}path_y: -215.0,")
lines.append(f"{INDENT}),")
lines.append(f"{INDENT}waves: [")

for i in range(len(waves_config)):
    delay = waves_config[i][0]
    entries = waves_config[i][1]
    INDENT4 = " " * 16
    lines.append(f"{INDENT2}// {'=' * 10} {WAVE_COMMENTS[i]} {'=' * 10}")
    lines.append(f"{INDENT2}(")
    lines.append(f"{INDENT3}delay: {fmt_num(delay)},")
    lines.append(f"{INDENT3}wave: [")
    for edelay, kind, count, interval in entries:
        lines.append(f"{INDENT4}(delay: {fmt_num(edelay)}, kind: {kind}, count: {count}, interval: {fmt_num(interval)}),")
    lines.append(f"{INDENT3}],")
    lines.append(f"{INDENT2}),")

lines.append(f"{INDENT}],")
lines.append(")")

output = "\n".join(lines)

# Overwrite the level file
import os
script_dir = os.path.dirname(os.path.abspath(__file__))
target = os.path.join(script_dir, "level_row_three_physics_line.ron")
with open(target, "w", encoding="utf-8") as f:
    f.write(output)
    f.write("\n")

print(f"\nWritten to: {target}", flush=True)
