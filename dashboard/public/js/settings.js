// Settings panel logic for Pumpkin Dashboard

const SettingsPanel = (() => {
  function init() {
    document.getElementById('settings-form').addEventListener('submit', handleSave);
  }

  function load(data) {
    if (data.motd !== undefined) {
      document.getElementById('setting-motd').value = data.motd;
    }
    if (data.max_players !== undefined) {
      document.getElementById('setting-max-players').value = data.max_players;
    }
    if (data.default_gamemode !== undefined) {
      document.getElementById('setting-gamemode').value = data.default_gamemode;
    }
    if (data.default_difficulty !== undefined) {
      document.getElementById('setting-difficulty').value = data.default_difficulty;
    }
  }

  async function handleSave(e) {
    e.preventDefault();

    const btn = document.getElementById('save-settings-btn');
    btn.disabled = true;
    btn.textContent = 'Saving...';

    const settings = {
      motd: document.getElementById('setting-motd').value,
      max_players: parseInt(document.getElementById('setting-max-players').value, 10),
      default_gamemode: document.getElementById('setting-gamemode').value,
      default_difficulty: document.getElementById('setting-difficulty').value
    };

    try {
      const res = await fetch('/api/settings', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(settings)
      });

      if (res.ok) {
        App.showToast('Settings saved successfully', 'success');
      } else {
        const data = await res.json();
        App.showToast(data.error || 'Failed to save settings', 'error');
      }
    } catch (err) {
      App.showToast('Failed to connect to server', 'error');
    } finally {
      btn.disabled = false;
      btn.textContent = 'Save Settings';
    }
  }

  // Initialize on DOM ready
  document.addEventListener('DOMContentLoaded', init);

  return { load };
})();
