// Main application logic for Pumpkin Dashboard

const App = (() => {
  let serverStatus = 'offline';
  let statusPollInterval = null;
  let playerPollInterval = null;

  function init() {
    fetchStatus();
    fetchPlayers();
    loadSettings();

    statusPollInterval = setInterval(fetchStatus, 3000);
    playerPollInterval = setInterval(fetchPlayers, 5000);

    document.getElementById('power-button').addEventListener('click', handlePowerClick);

    ConsolePanel.init();
  }

  async function fetchStatus() {
    try {
      const res = await fetch('/api/status');
      const data = await res.json();
      updateStatusUI(data);
    } catch (err) {
      updateStatusUI({ online: false });
    }
  }

  function updateStatusUI(data) {
    const powerBtn = document.getElementById('power-button');
    const statusText = document.getElementById('status-text');
    const uptimeEl = document.getElementById('uptime');

    powerBtn.classList.remove('online', 'offline', 'starting');
    statusText.classList.remove('online', 'offline', 'starting');

    if (data.online) {
      serverStatus = 'online';
      powerBtn.classList.add('online');
      statusText.classList.add('online');
      statusText.textContent = 'Online';
    } else {
      serverStatus = 'offline';
      powerBtn.classList.add('offline');
      statusText.classList.add('offline');
      statusText.textContent = 'Offline';
    }

    if (data.uptime) {
      uptimeEl.textContent = 'Uptime: ' + formatUptime(data.uptime);
    } else {
      uptimeEl.textContent = '';
    }

    // Update resources
    if (data.resources) {
      updateResources(data.resources);
    }

    // Update server address if available
    if (data.address) {
      document.getElementById('server-address').textContent = data.address;
    }
  }

  function updateResources(resources) {
    const cpuValue = document.getElementById('cpu-value');
    const cpuBar = document.getElementById('cpu-bar');
    const ramValue = document.getElementById('ram-value');
    const ramBar = document.getElementById('ram-bar');

    if (resources.cpu !== undefined) {
      const cpuPct = Math.round(resources.cpu);
      cpuValue.textContent = cpuPct + '%';
      cpuBar.style.width = cpuPct + '%';
    }

    if (resources.memory) {
      const usedMB = Math.round(resources.memory.used / (1024 * 1024));
      const totalMB = Math.round(resources.memory.total / (1024 * 1024));
      const pct = totalMB > 0 ? Math.round((resources.memory.used / resources.memory.total) * 100) : 0;
      ramValue.textContent = usedMB + ' / ' + totalMB + ' MB';
      ramBar.style.width = pct + '%';
    }
  }

  function formatUptime(seconds) {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    const parts = [];
    if (h > 0) parts.push(h + 'h');
    if (m > 0) parts.push(m + 'm');
    parts.push(s + 's');
    return parts.join(' ');
  }

  async function handlePowerClick() {
    if (serverStatus === 'online') {
      if (confirm('Are you sure you want to stop the server?')) {
        try {
          const res = await fetch('/api/power/stop', { method: 'POST' });
          const data = await res.json();
          if (res.ok) {
            showToast('Server stop command sent', 'success');
            setStartingState();
          } else {
            showToast(data.error || 'Failed to stop server', 'error');
          }
        } catch (err) {
          showToast('Failed to connect to dashboard API', 'error');
        }
      }
    } else {
      showToast('Server must be started externally', 'error');
    }
  }

  function setStartingState() {
    const powerBtn = document.getElementById('power-button');
    const statusText = document.getElementById('status-text');

    powerBtn.classList.remove('online', 'offline');
    powerBtn.classList.add('starting');
    statusText.classList.remove('online', 'offline');
    statusText.classList.add('starting');
    statusText.textContent = 'Stopping...';
    serverStatus = 'starting';
  }

  async function fetchPlayers() {
    try {
      const res = await fetch('/api/players');
      const data = await res.json();
      PlayersPanel.update(data);
    } catch (err) {
      PlayersPanel.update({ online: false });
    }
  }

  async function loadSettings() {
    try {
      const res = await fetch('/api/settings');
      const data = await res.json();
      SettingsPanel.load(data);
    } catch (err) {
      // Settings will use default placeholder values
    }
  }

  function showToast(message, type) {
    const container = document.getElementById('toast-container');
    const toast = document.createElement('div');
    toast.className = 'toast ' + type;
    toast.textContent = message;
    container.appendChild(toast);

    setTimeout(() => {
      toast.style.opacity = '0';
      toast.style.transform = 'translateX(20px)';
      toast.style.transition = 'all 0.3s ease';
      setTimeout(() => toast.remove(), 300);
    }, 3000);
  }

  return { init, showToast, fetchPlayers };
})();

document.addEventListener('DOMContentLoaded', App.init);
