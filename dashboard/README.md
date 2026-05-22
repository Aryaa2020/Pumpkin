# Pumpkin Dashboard

An Aternos-inspired web dashboard for managing your [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) Minecraft server. Monitor server status, manage players, view the live console, and configure server settings -- all from a modern web interface.

## Features

- **Server Start/Stop** - Large power button with real-time status indicator (online, offline, starting)
- **Real-time Console** - Live log streaming via WebSocket with command input and history
- **Player Management** - View online players with avatars, player count, and kick functionality
- **Server Settings** - Edit MOTD, max players, gamemode, and difficulty from the UI
- **Resource Monitoring** - View CPU and memory usage of the server

## Prerequisites

- **Node.js 18+** (Node.js 22 recommended)
- **Pumpkin Minecraft server** with RCON enabled

## Quick Start

```bash
cd dashboard
npm install
npm start
```

The dashboard will be available at [http://localhost:3000](http://localhost:3000).

## Configuration

The dashboard is configured via environment variables. You can set them in a `.env` file or pass them directly.

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `3000` | Port the dashboard web server listens on |
| `RCON_HOST` | `localhost` | Hostname of the Pumpkin server RCON interface |
| `RCON_PORT` | `25575` | Port of the Pumpkin server RCON interface |
| `RCON_PASSWORD` | (empty) | Password for RCON authentication |
| `CONFIG_PATH` | `../pumpkin.toml` | Path to the Pumpkin server configuration file |

Copy the example environment file to get started:

```bash
cp .env.example .env
```

## Pumpkin RCON Setup

The dashboard communicates with Pumpkin via RCON (Remote Console). You need to enable RCON in your Pumpkin server configuration.

Edit your `pumpkin.toml` and configure the `[networking.rcon]` section:

```toml
[networking.rcon]
enabled = true
address = "0.0.0.0:25575"
password = "your-secure-password"
```

Make sure the `RCON_PASSWORD` environment variable in the dashboard matches the password set in `pumpkin.toml`.

## Docker Usage

The easiest way to run the dashboard alongside Pumpkin is with Docker Compose:

```bash
docker-compose up -d
```

This starts both the Pumpkin server and the dashboard. The dashboard will be accessible at [http://localhost:3000](http://localhost:3000) and the Minecraft server at `localhost:25565`.

To set the RCON password when using Docker Compose, use an environment variable:

```bash
RCON_PASSWORD=your-secure-password docker-compose up -d
```

## Development

### Running in Development Mode

```bash
cd dashboard
npm install
npm run dev
```

This starts the server with `nodemon` for automatic restarts on file changes.

### Running Tests

```bash
npm test
```

### Project Structure

```
dashboard/
├── public/              # Static frontend files
│   ├── index.html       # Main dashboard page
│   ├── favicon.ico      # Site icon
│   ├── css/
│   │   └── style.css    # Dark theme styles
│   └── js/
│       ├── app.js       # Main application logic
│       ├── console.js   # Console panel logic
│       ├── players.js   # Player list panel
│       └── settings.js  # Settings panel
├── src/                 # Backend source
│   ├── server.js        # Express server entry point
│   ├── config.js        # Configuration loader
│   ├── rcon-client.js   # RCON protocol client
│   └── routes/          # API route handlers
│       ├── index.js     # Route registry
│       ├── console.js   # Console/command endpoints
│       ├── players.js   # Player management endpoints
│       ├── power.js     # Start/stop/restart endpoints
│       ├── settings.js  # Settings CRUD endpoints
│       └── status.js    # Server status endpoint
├── test/                # Test files
│   └── api.test.js      # API integration tests
├── Dockerfile           # Container build definition
├── .env.example         # Environment variable template
├── package.json         # Node.js project manifest
└── README.md            # This file
```

## License

Same as the parent Pumpkin project - [GPL-3.0](../LICENSE).
