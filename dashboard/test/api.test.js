const { describe, it, before, after } = require('node:test');
const assert = require('node:assert');
const http = require('http');

const { app, server } = require('../src/server');

let baseUrl;

function request(path, options = {}) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, baseUrl);
    const opts = {
      hostname: url.hostname,
      port: url.port,
      path: url.pathname,
      method: options.method || 'GET',
      headers: options.headers || {},
    };

    if (options.body) {
      opts.headers['Content-Type'] = 'application/json';
    }

    const req = http.request(opts, (res) => {
      let data = '';
      res.on('data', (chunk) => { data += chunk; });
      res.on('end', () => {
        try {
          resolve({ status: res.statusCode, body: JSON.parse(data) });
        } catch {
          resolve({ status: res.statusCode, body: data });
        }
      });
    });

    req.on('error', reject);

    if (options.body) {
      req.write(JSON.stringify(options.body));
    }
    req.end();
  });
}

describe('Dashboard API', () => {
  before((context, done) => {
    server.listen(0, () => {
      const addr = server.address();
      baseUrl = `http://localhost:${addr.port}`;
      done();
    });
  });

  after((context, done) => {
    server.close(done);
  });

  describe('GET /health', () => {
    it('should return ok status', async () => {
      const res = await request('/health');
      assert.strictEqual(res.status, 200);
      assert.strictEqual(res.body.status, 'ok');
    });
  });

  describe('GET /api/status', () => {
    it('should return status with system info', async () => {
      const res = await request('/api/status');
      assert.strictEqual(res.status, 200);
      assert.strictEqual(typeof res.body.online, 'boolean');
      assert.ok(res.body.system);
      assert.strictEqual(typeof res.body.system.cpuCount, 'number');
      assert.strictEqual(typeof res.body.system.totalMemory, 'number');
      assert.strictEqual(typeof res.body.system.freeMemory, 'number');
      assert.strictEqual(typeof res.body.system.usedMemory, 'number');
      assert.strictEqual(typeof res.body.system.memoryUsagePercent, 'string');
    });
  });

  describe('GET /api/players', () => {
    it('should return error when RCON is unavailable', async () => {
      const res = await request('/api/players');
      // Server is not running, so RCON will fail
      assert.strictEqual(res.status, 503);
      assert.ok(res.body.error);
      assert.strictEqual(typeof res.body.online, 'number');
      assert.strictEqual(typeof res.body.max, 'number');
      assert.ok(Array.isArray(res.body.players));
    });
  });

  describe('POST /api/console', () => {
    it('should reject missing command', async () => {
      const res = await request('/api/console', {
        method: 'POST',
        body: {},
      });
      assert.strictEqual(res.status, 400);
      assert.ok(res.body.error);
    });

    it('should return error when RCON is unavailable', async () => {
      const res = await request('/api/console', {
        method: 'POST',
        body: { command: 'list' },
      });
      assert.strictEqual(res.status, 503);
      assert.ok(res.body.error);
    });
  });

  describe('GET /api/settings', () => {
    it('should return settings object', async () => {
      const res = await request('/api/settings');
      assert.strictEqual(res.status, 200);
      assert.ok(res.body.settings);
      assert.strictEqual(typeof res.body.settings.motd, 'string');
      assert.strictEqual(typeof res.body.settings.max_players, 'number');
      assert.strictEqual(typeof res.body.settings.default_gamemode, 'string');
      assert.strictEqual(typeof res.body.settings.default_difficulty, 'string');
    });
  });

  describe('POST /api/power/stop', () => {
    it('should return error when RCON is unavailable', async () => {
      const res = await request('/api/power/stop', {
        method: 'POST',
      });
      assert.strictEqual(res.status, 503);
      assert.ok(res.body.error);
    });
  });
});
