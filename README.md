# CS2 Skin Changer

A standalone skin changer for Counter-Strike 2 on Linux. This is a minimal extraction of the skin changer feature from the deadlocked project.

## Features

- **Fallback Field System**: Uses CS2's fallback fields on `C_EconEntity` to override inventory data:
  - `m_nFallbackPaintKit`: Skin ID
  - `m_nFallbackSeed`: Pattern seed
  - `m_flFallbackWear`: Wear value (0.0 - 1.0)
  - `m_nFallbackStatTrak`: StatTrak kill count (-1 = disabled)

- **Dynamic Offset Discovery**: Automatically finds offsets at runtime using schema dumping
- **Persistent Application**: Continuously reapplies skins to handle round resets
- **All Weapons Supported**: Applies skins to all weapons in inventory, not just the active one

## Requirements

- Linux operating system
- CS2 running
- User must be in the `input` group (do NOT run as root)

## Building

```bash
cargo build --release
```

## Running

```bash
./target/release/cs2-skin-changer
```

**Important**: Do NOT run as root. Instead, add your user to the input group:

```bash
sudo usermod -aG input $USER
```

Then log out and back in for the change to take effect.

## Configuration

The config file is stored at `~/.config/cs2-skin-changer/cs2-skin-changer.toml`

Example configuration:

```toml
enabled = true

[skins.ak47]
enabled = true
paint_kit = 675  # Asiimov
seed = 0
wear = 0.0
stattrak = -1

[skins.awp]
enabled = true
paint_kit = 344  # Dragon Lore
seed = 0
wear = 0.0
stattrak = 100

[skins.m4a4]
enabled = true
paint_kit = 309  # Howl
seed = 0
wear = 0.0
stattrak = -1
```

### Paint Kit IDs

Some popular paint kit IDs:

- **AK-47**:
  - Asiimov: 675
  - Fire Serpent: 180
  - Case Hardened: 44
  - Vulcan: 302

- **AWP**:
  - Dragon Lore: 344
  - Medusa: 446
  - Asiimov: 279

- **M4A4**:
  - Howl: 309
  - Asiimov: 471

- **M4A1-S**:
  - Hyper Beast: 430
  - Hot Rod: 445

- **Knife**:
  - Case Hardened: 44
  - Fade: 38
  - Tiger Tooth: 409

## How It Works

1. Opens the CS2 process and finds required memory offsets dynamically
2. Locates the local player and their weapon inventory
3. For each weapon with an enabled skin:
   - Sets `m_iItemIDHigh` to -1 to force fallback usage
   - Writes fallback paint kit, seed, wear, and stattrak values
   - Sets ownership fields to prevent reset
4. Repeats every ~10ms to catch game resets

## License

Same license as the original deadlocked project.
