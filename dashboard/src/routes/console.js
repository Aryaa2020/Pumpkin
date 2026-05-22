const express = require('express');
const { createRconClient } = require('../rcon-client');
const config = require('../config');

const router = express.Router();

router.post('/', async (req, res) => {
  const { command } = req.body;

  if (!command || typeof command !== 'string') {
    return res.status(400).json({
      error: 'Invalid request',
      message: 'A "command" string is required in the request body',
    });
  }

  let client = null;

  try {
    client = await createRconClient(config.rconHost, config.rconPort, config.rconPassword);
    const response = await client.sendCommand(command);
    client.disconnect();

    res.json({
      command,
      response,
    });
  } catch (err) {
    res.status(503).json({
      error: 'Unable to execute command',
      message: err.message,
    });
  } finally {
    if (client) {
      client.disconnect();
    }
  }
});

module.exports = router;
