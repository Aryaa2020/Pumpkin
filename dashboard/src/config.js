const path = require('path');

const config = {
  rconHost: process.env.RCON_HOST || 'localhost',
  rconPort: parseInt(process.env.RCON_PORT, 10) || 25575,
  rconPassword: process.env.RCON_PASSWORD || '',
  configPath: process.env.CONFIG_PATH || path.resolve(__dirname, '../../pumpkin.toml'),
  port: parseInt(process.env.PORT, 10) || 3000,
  dashboardToken: process.env.DASHBOARD_TOKEN || '',
};

module.exports = config;
