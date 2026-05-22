const express = require('express');
const { createRconClient } = require('../rcon-client');
const config = require('../config');

const router = express.Router();

router.post('/stop', async (req, res) => {
  let client = null;

  try {
    client = await createRconClient(config.rconHost, config.rconPort, config.rconPassword);
    const response = await client.sendCommand('stop');
    client.disconnect();

    res.json({
      success: true,
      message: 'Stop command sent',
      response,
    });
  } catch (err) {
    res.status(503).json({
      error: 'Unable to send stop command',
      message: err.message,
    });
  } finally {
    if (client) {
      client.disconnect();
    }
  }
});

module.exports = router;
