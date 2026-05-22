const express = require('express');
const { createRconClient } = require('../rcon-client');
const config = require('../config');

const router = express.Router();

router.get('/', async (req, res) => {
  let client = null;

  try {
    client = await createRconClient(config.rconHost, config.rconPort, config.rconPassword);
    const response = await client.sendCommand('list');
    client.disconnect();

    // Parse response like: "There are X of a max of Y players online: player1, player2"
    const match = response.match(/There are (\d+) of a max of (\d+) players online:(.*)/i);
    let online = 0;
    let max = 0;
    let players = [];

    if (match) {
      online = parseInt(match[1], 10);
      max = parseInt(match[2], 10);
      const playerStr = match[3].trim();
      if (playerStr.length > 0) {
        players = playerStr.split(',').map((p) => p.trim()).filter((p) => p.length > 0);
      }
    }

    res.json({
      online,
      max,
      players,
    });
  } catch (err) {
    res.status(503).json({
      error: 'Unable to connect to server',
      message: err.message,
      online: 0,
      max: 0,
      players: [],
    });
  } finally {
    if (client) {
      client.disconnect();
    }
  }
});

module.exports = router;
