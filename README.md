# tdate - Thelemic Date Calculator

A Rust implementation of the Thelemic Date system, providing current Thelemic date information including solar and lunar positions.

## Features

- Displays the current Thelemic date in the format: `☉ in Xº Sign : ☽ in Yº Sign : dies Day : Anno Year æræ legis`
- Calculates precise solar and lunar positions using astronomical algorithms
- Automatically determines timezone based on location
- Uses the Thelemic calendar system (New Year begins at the vernal equinox)

## Installation

### Prerequisites
- Rust 1.82 or later

### Building from source

```bash
git clone https://github.com/JSKitty/tdate.git
cd tdate
cargo build --release
```

The executable will be available at `target/release/tdate`

## Usage

```bash
# Display current Thelemic date (default location: Las Vegas, NV)
tdate

# Display current date for a specific location
tdate --location "London, UK"
tdate -l "Tokyo, Japan"

# Display a specific date and time
tdate 2024 3 20 12 0 "New York, NY"
```

### Options

- `-h, --help` - Print help information
- `-V, --version` - Print version information  
- `-l, --location <LOCATION>` - Specify location for date calculation (e.g., "Las Vegas, NV")

### Example output:
```
☉ in 1º Leo : ☽ in 16º Cancer : dies Mercurii : Anno Vxi æræ legis
```

## Implementation Details

This is a Rust port of the original Python implementation, maintaining exact 1:1 logic with the original while leveraging Rust's performance and type safety.

The program uses the following crates:
- `astro` - For astronomical calculations
- `chrono` & `chrono-tz` - For date/time handling and timezone support
- `geocoding` - For location-based timezone determination
- `tzf-rs` - For timezone finding based on coordinates
- `clap` - For command-line argument parsing

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Original Implementation

Based on the Python implementation by Lilith Vala Xara: https://github.com/LilithXara/ThelemicDate
