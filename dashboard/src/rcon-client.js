const net = require('net');

class RconClient {
  constructor(host, port, password) {
    this.host = host;
    this.port = port;
    this.password = password;
    this.socket = null;
    this.requestId = 0;
    this.connected = false;
    this.authenticated = false;
  }

  _nextRequestId() {
    this.requestId = (this.requestId + 1) & 0x7fffffff;
    return this.requestId;
  }

  _encodePacket(requestId, type, body) {
    const bodyBuffer = Buffer.from(body, 'utf8');
    // length = 4 (requestId) + 4 (type) + body length + 1 (null terminator) + 1 (padding)
    const length = 4 + 4 + bodyBuffer.length + 1 + 1;
    const packet = Buffer.alloc(4 + length);
    packet.writeInt32LE(length, 0);
    packet.writeInt32LE(requestId, 4);
    packet.writeInt32LE(type, 8);
    bodyBuffer.copy(packet, 12);
    packet[12 + bodyBuffer.length] = 0; // null terminator
    packet[12 + bodyBuffer.length + 1] = 0; // padding
    return packet;
  }

  _decodePacket(buffer) {
    if (buffer.length < 14) {
      return null;
    }
    const length = buffer.readInt32LE(0);
    if (buffer.length < 4 + length) {
      return null;
    }
    const requestId = buffer.readInt32LE(4);
    const type = buffer.readInt32LE(8);
    const body = buffer.slice(12, 4 + length - 2).toString('utf8');
    return { length, requestId, type, body, totalSize: 4 + length };
  }

  connect() {
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        if (this.socket) {
          this.socket.destroy();
        }
        reject(new Error('Connection timeout'));
      }, 5000);

      this.socket = new net.Socket();

      this.socket.on('error', (err) => {
        clearTimeout(timeout);
        this.connected = false;
        this.authenticated = false;
        reject(err);
      });

      this.socket.connect(this.port, this.host, () => {
        clearTimeout(timeout);
        this.connected = true;
        resolve();
      });
    });
  }

  authenticate() {
    return new Promise((resolve, reject) => {
      if (!this.connected) {
        return reject(new Error('Not connected'));
      }

      const reqId = this._nextRequestId();
      const packet = this._encodePacket(reqId, 3, this.password);

      const timeout = setTimeout(() => {
        reject(new Error('Authentication timeout'));
      }, 5000);

      let dataBuffer = Buffer.alloc(0);

      const onData = (data) => {
        dataBuffer = Buffer.concat([dataBuffer, data]);
        const response = this._decodePacket(dataBuffer);
        if (response) {
          clearTimeout(timeout);
          this.socket.removeListener('data', onData);
          if (response.requestId === -1) {
            reject(new Error('Authentication failed'));
          } else {
            this.authenticated = true;
            resolve();
          }
        }
      };

      this.socket.on('data', onData);
      this.socket.write(packet);
    });
  }

  sendCommand(command) {
    return new Promise((resolve, reject) => {
      if (!this.connected || !this.authenticated) {
        return reject(new Error('Not connected or not authenticated'));
      }

      const reqId = this._nextRequestId();
      const packet = this._encodePacket(reqId, 2, command);

      const timeout = setTimeout(() => {
        this.socket.removeListener('data', onData);
        reject(new Error('Command timeout'));
      }, 10000);

      let dataBuffer = Buffer.alloc(0);

      const onData = (data) => {
        dataBuffer = Buffer.concat([dataBuffer, data]);
        const response = this._decodePacket(dataBuffer);
        if (response) {
          clearTimeout(timeout);
          this.socket.removeListener('data', onData);
          resolve(response.body);
        }
      };

      this.socket.on('data', onData);
      this.socket.write(packet);
    });
  }

  disconnect() {
    if (this.socket) {
      this.socket.destroy();
      this.socket = null;
    }
    this.connected = false;
    this.authenticated = false;
  }
}

async function createRconClient(host, port, password) {
  const client = new RconClient(host, port, password);
  await client.connect();
  await client.authenticate();
  return client;
}

module.exports = { RconClient, createRconClient };
