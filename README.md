# Solis Delay

A Rust application that intelligently manages battery charging for Solis inverters to reduce grid impact after blackouts and prevent rapid charging cycles through hysteresis.

## Overview

After a power blackout, when the grid is restored, many homes and businesses simultaneously begin charging their batteries, creating a sudden surge in grid demand. This application addresses this problem by **delaying grid charging after blackout recovery**: when the grid comes back online, the application waits a configurable delay period before allowing the inverter to charge from the grid. This staggers the charging load and reduces the immediate impact on the grid.

**Implementing hysteresis for battery charging**: The application uses a min/max battery percentage system to prevent rapid on/off cycling of grid charging. Once charging stops at the maximum threshold, it won't resume until the battery drops to the minimum threshold.

## How It Works

The application continuously monitors your Solis inverter (every 30 seconds) and:

- Detects grid status by monitoring AC voltage
- Tracks when the grid comes back online after a blackout
- Enforces a delay period before allowing grid charging to resume
- Controls the "allow grid charging" parameter on the inverter based on:
  - Grid availability
  - Time since grid restoration
  - Current battery percentage
  - Hysteresis thresholds

### Charging Logic

Grid charging is allowed when **all** of the following conditions are met:

1. Grid is active (AC voltage > 2V)
2. Sufficient time has passed since grid restoration (configurable delay)
3. Battery percentage is below the maximum threshold
4. Either:
   - Charging was already enabled, OR
   - Battery percentage is below the minimum threshold (hysteresis)

## Installation

### Prerequisites

- Rust (latest stable version)
- Access to Solis Cloud API credentials

### Build

```bash
cargo build --release
```

The optimized binary will be in `target/release/solis-delay`.

### Docker

The application can also be run using Docker with a multi-stage build for optimal image size.

#### Build the Docker Image

```bash
docker build -t solis-delay .
```

#### Run with Docker

You can pass environment variables directly or use a `.env` file:

**Using environment variables:**

```bash
docker run -d \
  --name solis-delay \
  --restart unless-stopped \
  -e SOLIS_API_URL=https://api.soliscloud.com \
  -e SOLIS_API_KEY_ID=your_key_id \
  -e SOLIS_API_KEY_SECRET=your_key_secret \
  -e SOLIS_INVERTER_SN=1234567890 \
  -e SOLIS_GRID_CHARGING_DELAY=600 \
  -e SOLIS_MIN_BATTERY_PERCENT=90 \
  -e SOLIS_MAX_BATTERY_PERCENT=95 \
  -e RUST_LOG=info \
  solis-delay
```

**Using a `.env` file:**

```bash
docker run -d \
  --name solis-delay \
  --restart unless-stopped \
  --env-file .env \
  -e RUST_LOG=info \
  solis-delay
```

**First run (discover inverters):**

If you need to discover your inverter serial number, run without `SOLIS_INVERTER_SN`:

```bash
docker run --rm \
  --env-file .env \
  solis-delay
```

The container will list available inverters and exit.

#### View Logs

```bash
docker logs -f solis-delay
```

## Configuration

Create a `.env` file in the project root with the following variables:

### Required

- `SOLIS_API_URL` - Your Solis API endpoint URL (e.g., `https://api.soliscloud.com`)
- `SOLIS_API_KEY_ID` - Your Solis API key ID
- `SOLIS_API_KEY_SECRET` - Your Solis API key secret

### Optional

- `SOLIS_INVERTER_SN` - Your inverter serial number. If not set, the application will list available inverters and exit.
- `SOLIS_GRID_CHARGING_DELAY` - Delay in seconds before allowing grid charging after grid restoration (default: `600` = 10 minutes)
- `SOLIS_MIN_BATTERY_PERCENT` - Minimum battery percentage to start charging (default: `90`)
- `SOLIS_MAX_BATTERY_PERCENT` - Maximum battery percentage to stop charging (default: `95`)

### Example `.env` file

```env
SOLIS_API_URL=https://api.soliscloud.com
SOLIS_API_KEY_ID=your_key_id
SOLIS_API_KEY_SECRET=your_key_secret
SOLIS_INVERTER_SN=1234567890
SOLIS_GRID_CHARGING_DELAY=600
SOLIS_MIN_BATTERY_PERCENT=90
SOLIS_MAX_BATTERY_PERCENT=95
```

## Usage

### First Run (Discover Inverters)

If you don't know your inverter serial number, run without setting `SOLIS_INVERTER_SN`:

```bash
./target/release/solis-delay
```

The application will list all available inverters and exit. Copy the serial number and add it to your `.env` file.

### Normal Operation

Once configured, simply run:

```bash
./target/release/solis-delay
```

The application will run continuously, monitoring your inverter and automatically managing grid charging. It logs its actions and state changes for monitoring purposes.

### Logging

The application uses the `log` crate. Set the `RUST_LOG` environment variable to control logging levels:

```bash
RUST_LOG=info ./target/release/solis-delay
RUST_LOG=debug ./target/release/solis-delay  # More verbose
```

## Testing

Run the test suite:

```bash
make test
# or
cargo test
```
