const express = require('express');
const fs = require('fs').promises;
const TOML = require('@iarna/toml');
const config = require('../config');

const router = express.Router();

const DEFAULT_SETTINGS = {
  motd: 'A Minecraft Server powered by Pumpkin',
  max_players: 20,
  default_gamemode: 'Survival',
  default_difficulty: 'Normal',
};

const VALID_GAMEMODES = ['Survival', 'Creative', 'Adventure', 'Spectator'];
const VALID_DIFFICULTIES = ['Peaceful', 'Easy', 'Normal', 'Hard'];
const ALLOWED_FIELDS = ['motd', 'max_players', 'default_gamemode', 'default_difficulty'];

async function readConfig() {
  try {
    const content = await fs.readFile(config.configPath, 'utf8');
    return TOML.parse(content);
  } catch (err) {
    return null;
  }
}

async function writeConfig(data) {
  const content = TOML.stringify(data);
  await fs.writeFile(config.configPath, content, 'utf8');
}

function validateSettings(body) {
  const errors = [];

  // Reject unknown keys
  const unknownKeys = Object.keys(body).filter((key) => !ALLOWED_FIELDS.includes(key));
  if (unknownKeys.length > 0) {
    errors.push(`Unknown fields: ${unknownKeys.join(', ')}`);
  }

  if (body.motd !== undefined) {
    if (typeof body.motd !== 'string') {
      errors.push('motd must be a string');
    } else if (body.motd.length > 256) {
      errors.push('motd must be at most 256 characters');
    }
  }

  if (body.max_players !== undefined) {
    if (!Number.isInteger(body.max_players) || body.max_players < 1 || body.max_players > 10000) {
      errors.push('max_players must be a positive integer between 1 and 10000');
    }
  }

  if (body.default_gamemode !== undefined) {
    if (!VALID_GAMEMODES.includes(body.default_gamemode)) {
      errors.push(`default_gamemode must be one of: ${VALID_GAMEMODES.join(', ')}`);
    }
  }

  if (body.default_difficulty !== undefined) {
    if (!VALID_DIFFICULTIES.includes(body.default_difficulty)) {
      errors.push(`default_difficulty must be one of: ${VALID_DIFFICULTIES.join(', ')}`);
    }
  }

  return errors;
}

router.get('/', async (req, res) => {
  try {
    const tomlData = await readConfig();

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

router.put('/', async (req, res) => {
  try {
    const errors = validateSettings(req.body);
    if (errors.length > 0) {
      return res.status(400).json({
        error: 'Validation failed',
        messages: errors,
      });
    }

    const { motd, max_players, default_gamemode, default_difficulty } = req.body;

    let tomlData = (await readConfig()) || {};

    if (motd !== undefined) tomlData.motd = motd;
    if (max_players !== undefined) tomlData.max_players = max_players;
    if (default_gamemode !== undefined) tomlData.default_gamemode = default_gamemode;
    if (default_difficulty !== undefined) tomlData.default_difficulty = default_difficulty;

    await writeConfig(tomlData);

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
