// Players panel logic for Pumpkin Dashboard

const PlayersPanel = (() => {
  function update(data) {
    const listEl = document.getElementById('player-list');
    const countEl = document.getElementById('player-count');

    if (data.online === false || data.error) {
      countEl.textContent = '0/0';
      listEl.innerHTML = '<div class="empty-message">Server offline</div>';
      return;
    }

    const players = data.players || [];
    const onlineCount = data.online_count !== undefined ? data.online_count : players.length;
    const maxCount = data.max || 20;

    countEl.textContent = onlineCount + '/' + maxCount;

    if (players.length === 0) {
      listEl.innerHTML = '<div class="empty-message">No players online</div>';
      return;
    }

    listEl.innerHTML = '';
    players.forEach(player => {
      const entry = document.createElement('div');
      entry.className = 'player-entry';

      const avatar = document.createElement('img');
      avatar.className = 'player-avatar';
      const uuid = player.uuid || '00000000-0000-0000-0000-000000000000';
      avatar.src = 'https://crafatar.com/avatars/' + uuid + '?size=32&overlay';
      avatar.alt = player.name;
      avatar.onerror = function() {
        this.src = 'data:image/svg+xml,' + encodeURIComponent(
          '<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><rect fill="#333" width="32" height="32"/><text x="16" y="20" text-anchor="middle" fill="#aaa" font-size="14">' + (player.name ? player.name[0].toUpperCase() : '?') + '</text></svg>'
        );
      };

      const name = document.createElement('span');
      name.className = 'player-name';
      name.textContent = player.name || 'Unknown';

      const kickBtn = document.createElement('button');
      kickBtn.className = 'player-kick-btn';
      kickBtn.textContent = 'Kick';
      kickBtn.addEventListener('click', () => kickPlayer(player.name));

      entry.appendChild(avatar);
      entry.appendChild(name);
      entry.appendChild(kickBtn);
      listEl.appendChild(entry);
    });
  }

  async function kickPlayer(playerName) {
    if (!confirm('Kick player ' + playerName + '?')) return;

    try {
      const res = await fetch('/api/console', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command: 'kick ' + playerName })
      });

      if (res.ok) {
        App.showToast('Kicked ' + playerName, 'success');
        // Refresh player list after a short delay
        setTimeout(() => App.fetchPlayers(), 1000);
      } else {
        const data = await res.json();
        App.showToast(data.error || 'Failed to kick player', 'error');
      }
    } catch (err) {
      App.showToast('Failed to send kick command', 'error');
    }
  }

  return { update };
})();
