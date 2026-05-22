// Console panel logic for Pumpkin Dashboard

const ConsolePanel = (() => {
  let ws = null;
  let commandHistory = [];
  let historyIndex = -1;
  let autoScroll = true;
  let reconnectTimer = null;

  function init() {
    const input = document.getElementById('console-input');
    const output = document.getElementById('console-output');

    input.addEventListener('keydown', handleKeyDown);

    output.addEventListener('scroll', () => {
      const threshold = 20;
      const atBottom = output.scrollHeight - output.scrollTop - output.clientHeight < threshold;
      autoScroll = atBottom;
    });

    connectWebSocket();
  }

  function connectWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = protocol + '//' + window.location.host + '/ws/console';

    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      appendLine('Connected to console', 'info');
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.error) {
          appendLine('[ERROR] ' + data.error, 'error');
        } else if (data.response !== undefined) {
          appendLine(data.response || '(empty response)', 'info');
        } else if (data.log) {
          appendLine(data.log, classifyLine(data.log));
        }
      } catch (err) {
        appendLine(event.data, 'info');
      }
    };

    ws.onclose = () => {
      appendLine('Disconnected from console', 'warn');
      scheduleReconnect();
    };

    ws.onerror = () => {
      // onclose will fire after this
    };
  }

  function scheduleReconnect() {
    if (reconnectTimer) return;
    reconnectTimer = setTimeout(() => {
      reconnectTimer = null;
      connectWebSocket();
    }, 5000);
  }

  function handleKeyDown(e) {
    const input = document.getElementById('console-input');

    if (e.key === 'Enter') {
      e.preventDefault();
      const command = input.value.trim();
      if (command) {
        sendCommand(command);
        addToHistory(command);
        input.value = '';
        historyIndex = -1;
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      navigateHistory(1);
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      navigateHistory(-1);
    }
  }

  function navigateHistory(direction) {
    const input = document.getElementById('console-input');
    if (commandHistory.length === 0) return;

    historyIndex += direction;

    if (historyIndex < 0) {
      historyIndex = -1;
      input.value = '';
      return;
    }

    if (historyIndex >= commandHistory.length) {
      historyIndex = commandHistory.length - 1;
    }

    input.value = commandHistory[historyIndex];
  }

  function addToHistory(command) {
    commandHistory.unshift(command);
    if (commandHistory.length > 50) {
      commandHistory.pop();
    }
  }

  async function sendCommand(command) {
    appendLine('> ' + command, 'info');

    try {
      const res = await fetch('/api/console', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command })
      });
      const data = await res.json();

      if (res.ok) {
        if (data.response) {
          appendLine(data.response, 'info');
        }
      } else {
        appendLine('[ERROR] ' + (data.error || 'Command failed'), 'error');
      }
    } catch (err) {
      appendLine('[ERROR] Failed to send command', 'error');
    }
  }

  function appendLine(text, className) {
    const output = document.getElementById('console-output');
    const line = document.createElement('div');
    line.className = 'console-line ' + (className || '');

    const timestamp = document.createElement('span');
    timestamp.className = 'timestamp';
    const now = new Date();
    timestamp.textContent = '[' + pad(now.getHours()) + ':' + pad(now.getMinutes()) + ':' + pad(now.getSeconds()) + ']';

    line.appendChild(timestamp);
    line.appendChild(document.createTextNode(text));
    output.appendChild(line);

    if (autoScroll) {
      output.scrollTop = output.scrollHeight;
    }
  }

  function classifyLine(text) {
    if (text.includes('ERROR')) return 'error';
    if (text.includes('WARN')) return 'warn';
    return 'info';
  }

  function pad(n) {
    return n < 10 ? '0' + n : '' + n;
  }

  return { init };
})();
