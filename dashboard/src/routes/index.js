const express = require('express');
const statusRouter = require('./status');
const playersRouter = require('./players');
const consoleRouter = require('./console');
const settingsRouter = require('./settings');
const powerRouter = require('./power');

const router = express.Router();

router.use('/status', statusRouter);
router.use('/players', playersRouter);
router.use('/console', consoleRouter);
router.use('/settings', settingsRouter);
router.use('/power', powerRouter);

module.exports = router;
