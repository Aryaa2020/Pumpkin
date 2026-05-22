const express = require('express');
const os = require('os');
const { createRconClient } = require('../rcon-client');
const config = require('../config');

const router = express.Router();

router.get('/', async (req, res) => {
  let serverOnline = false;
  let client = null;

  try {
    client = await createRconClient(config.rconHost, config.rconPort, config.rconPassword);
    serverOnline = true;
    client.disconnect();
  } catch (err) {
    serverOnline = false;
  }

  const cpus = os.cpus();
  const totalMem = os.totalmem();
  const freeMem = os.freemem();

  res.json({
    online: serverOnline,
    system: {
      cpuCount: cpus.length,
      cpuModel: cpus.length > 0 ? cpus[0].model : 'Unknown',
      totalMemory: totalMem,
      freeMemory: freeMem,
      usedMemory: totalMem - freeMem,
      memoryUsagePercent: ((totalMem - freeMem) / totalMem * 100).toFixed(1),
    },
  });
});

module.exports = router;
