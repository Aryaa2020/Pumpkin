const express = require('express');
const http = require('http');
const path = require('path');
const cors = require('cors');
const { WebSocketServer } = require('ws');
const config = require('./config');
const apiRouter = require('./routes');
const { createRconClient } = require('./rcon-client');

const app = express();
const server = http.createServer(app);

// Middleware
app.use(cors());
app.use(express.json());

// Serve static files from public/
app.use(express.static(path.join(__dirname, '../public')));

// Authentication middleware for API routes
function authMiddleware(req, res, next) {
  if (!config.dashboardToken) {
    return next();
  }

  const authHeader = req.headers.authorization;
  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return res.status(401).json({ error: 'Unauthorized: Bearer token required' });
  }

  const token = authHeader.slice(7);
  if (token !== config.dashboardToken) {
    return res.status(403).json({ error: 'Forbidden: Invalid token' });
  }

  next();
}

// API routes
app.use('/api', authMiddleware, apiRouter);

// Health check
app.get('/health', (req, res) => {
  res.json({ status: 'ok' });
});

// WebSocket server for console streaming
const wss = new WebSocketServer({ server, path: '/ws/console' });

wss.on('connection', (ws) => {
  ws.on('message', async (message) => {
    let data;
    try {
      data = JSON.parse(message.toString());
    } catch (err) {
      ws.send(JSON.stringify({ error: 'Invalid JSON' }));
      return;
    }

    if (!data.command || typeof data.command !== 'string') {
      ws.send(JSON.stringify({ error: 'A "command" string is required' }));
      return;
    }

    let client = null;
    try {
      client = await createRconClient(config.rconHost, config.rconPort, config.rconPassword);
      const response = await client.sendCommand(data.command);
      client.disconnect();
      ws.send(JSON.stringify({ command: data.command, response }));
    } catch (err) {
      ws.send(JSON.stringify({ error: err.message }));
    } finally {
      if (client) {
        client.disconnect();
      }
    }
  });
});

// Start server only if this file is run directly
if (require.main === module) {
  if (!config.rconPassword) {
    console.warn('WARNING: RCON_PASSWORD is not set. RCON authentication may fail or be insecure.');
  }
  if (!config.dashboardToken) {
    console.warn('WARNING: DASHBOARD_TOKEN is not set. The dashboard is running without authentication.');
  }
  server.listen(config.port, () => {
    console.log(`Pumpkin Dashboard running on http://localhost:${config.port}`);
  });
}

module.exports = { app, server };
