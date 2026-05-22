const express = require('express');
const fs = require('fs');
const TOML = require('@iarna/toml');
const config = require('../config');

const router = express.Router();

const DEFAULT_SETTINGS = {
  motd: 'A Minecraft Server powered by Pumpkin',
  max_players: 20,
  default_gamemode: 'Survival',
  default_difficulty: 'Normal',
};

function readConfig() {
  try {
    const content = fs.readFileSync(config.configPath, 'utf8');
    return TOML.parse(content);
  } catch (err) {
    return null;
  }
}

function writeConfig(data) {
  const content = TOML.stringify(data);
  fs.writeFileSync(config.configPath, content, 'utf8');
}

router.get('/', (req, res) => {
  try {
    const tomlData = readConfig();

    if (!tomlData) {
      return res.json({
        settings: { ...DEFAULT_SETTINGS },
        source: 'defaults',
      });
    }

    const settings = {
      motd: tomlData.motd || DEFAULT_SETTINGS.motd,
      max_players: tomlData.max_players || DEFAULT_SETTINGS.max_players,
      default_gamemode: tomlData.default_gamemode || DEFAULT_SETTINGS.default_gamemode,
      default_difficulty: tomlData.default_difficulty || DEFAULT_SETTINGS.default_difficulty,
    };

    res.json({
      settings,
      source: 'pumpkin.toml',
    });
  } catch (err) {
    res.status(500).json({
      error: 'Failed to read settings',
      message: err.message,
    });
  }
});

router.put('/', (req, res) => {
  try {
    const { motd, max_players, default_gamemode, default_difficulty } = req.body;

    let tomlData = readConfig() || {};

    if (motd !== undefined) tomlData.motd = motd;
    if (max_players !== undefined) tomlData.max_players = max_players;
    if (default_gamemode !== undefined) tomlData.default_gamemode = default_gamemode;
    if (default_difficulty !== undefined) tomlData.default_difficulty = default_difficulty;

    writeConfig(tomlData);

    res.json({
      success: true,
      settings: {
        motd: tomlData.motd || DEFAULT_SETTINGS.motd,
        max_players: tomlData.max_players || DEFAULT_SETTINGS.max_players,
        default_gamemode: tomlData.default_gamemode || DEFAULT_SETTINGS.default_gamemode,
        default_difficulty: tomlData.default_difficulty || DEFAULT_SETTINGS.default_difficulty,
      },
    });
  } catch (err) {
    res.status(500).json({
      error: 'Failed to update settings',
      message: err.message,
    });
  }
});

module.exports = router;
