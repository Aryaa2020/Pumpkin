/**
 * Reusable UI components for the Pumpkin Dashboard.
 */

const Components = {
    /**
     * Create a stat card widget.
     */
    statCard: function(label, value, colorClass) {
        const cls = colorClass ? ' ' + colorClass : '';
        return '<div class="stat-card">' +
            '<div class="stat-label">' + label + '</div>' +
            '<div class="stat-value' + cls + '">' + value + '</div>' +
            '</div>';
    },

    /**
     * Create a status badge (online/offline).
     */
    statusBadge: function(online) {
        const cls = online ? 'online' : 'offline';
        const text = online ? 'Online' : 'Offline';
        return '<span class="status-badge ' + cls + '">' +
            '<span class="status-dot"></span>' + text +
            '</span>';
    },

    /**
     * Create the player table HTML.
     */
    playerTable: function(players) {
        if (!players || players.length === 0) {
            return '<div class="table-container"><p style="padding:2rem;text-align:center;color:var(--text-muted)">No players online</p></div>';
        }
        var html = '<div class="table-container"><table>' +
            '<thead><tr><th>Name</th><th>UUID</th><th>Ping</th><th>World</th><th>Gamemode</th><th>Actions</th></tr></thead>' +
            '<tbody>';
        for (var i = 0; i < players.length; i++) {
            var p = players[i];
            html += '<tr>' +
                '<td>' + escapeHtml(p.name) + '</td>' +
                '<td style="font-family:monospace;font-size:0.75rem">' + escapeHtml(p.uuid) + '</td>' +
                '<td>' + p.ping + 'ms</td>' +
                '<td>' + escapeHtml(p.world) + '</td>' +
                '<td>' + escapeHtml(p.gamemode) + '</td>' +
                '<td>' +
                    '<button class="btn btn-danger btn-sm" onclick="App.kickPlayer(\'' + p.uuid + '\')">Kick</button> ' +
                    '<button class="btn btn-danger btn-sm" onclick="App.banPlayer(\'' + p.uuid + '\')">Ban</button>' +
                '</td></tr>';
        }
        html += '</tbody></table></div>';
        return html;
    },

    /**
     * Draw a simple TPS line chart on a canvas element.
     */
    drawTpsChart: function(canvasId, data) {
        var canvas = document.getElementById(canvasId);
        if (!canvas) return;
        var ctx = canvas.getContext('2d');
        var w = canvas.width = canvas.offsetWidth;
        var h = canvas.height = canvas.offsetHeight;

        ctx.clearRect(0, 0, w, h);

        if (!data || data.length < 2) return;

        var max = 20;
        var padding = 30;
        var graphW = w - padding * 2;
        var graphH = h - padding * 2;

        // Grid lines
        ctx.strokeStyle = '#3a3c5e';
        ctx.lineWidth = 1;
        for (var i = 0; i <= 4; i++) {
            var y = padding + (graphH / 4) * i;
            ctx.beginPath();
            ctx.moveTo(padding, y);
            ctx.lineTo(w - padding, y);
            ctx.stroke();
        }

        // Labels
        ctx.fillStyle = '#9a9ab5';
        ctx.font = '10px sans-serif';
        ctx.textAlign = 'right';
        for (var i = 0; i <= 4; i++) {
            var y = padding + (graphH / 4) * i;
            var val = max - (max / 4) * i;
            ctx.fillText(val.toFixed(0), padding - 5, y + 3);
        }

        // Draw line
        ctx.strokeStyle = '#f59e0b';
        ctx.lineWidth = 2;
        ctx.beginPath();
        for (var i = 0; i < data.length; i++) {
            var x = padding + (graphW / (data.length - 1)) * i;
            var y = padding + graphH - (data[i] / max) * graphH;
            if (i === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
        }
        ctx.stroke();

        // Fill area below
        ctx.lineTo(padding + graphW, padding + graphH);
        ctx.lineTo(padding, padding + graphH);
        ctx.closePath();
        ctx.fillStyle = 'rgba(245, 158, 11, 0.1)';
        ctx.fill();
    },

    /**
     * Draw a memory usage chart.
     */
    drawMemoryChart: function(canvasId, data, maxMem) {
        var canvas = document.getElementById(canvasId);
        if (!canvas) return;
        var ctx = canvas.getContext('2d');
        var w = canvas.width = canvas.offsetWidth;
        var h = canvas.height = canvas.offsetHeight;

        ctx.clearRect(0, 0, w, h);
        if (!data || data.length < 2) return;

        var max = maxMem || Math.max.apply(null, data) * 1.2;
        var padding = 30;
        var graphW = w - padding * 2;
        var graphH = h - padding * 2;

        // Grid
        ctx.strokeStyle = '#3a3c5e';
        ctx.lineWidth = 1;
        for (var i = 0; i <= 4; i++) {
            var y = padding + (graphH / 4) * i;
            ctx.beginPath();
            ctx.moveTo(padding, y);
            ctx.lineTo(w - padding, y);
            ctx.stroke();
        }

        // Labels
        ctx.fillStyle = '#9a9ab5';
        ctx.font = '10px sans-serif';
        ctx.textAlign = 'right';
        for (var i = 0; i <= 4; i++) {
            var y = padding + (graphH / 4) * i;
            var val = max - (max / 4) * i;
            ctx.fillText(val.toFixed(0) + 'MB', padding - 5, y + 3);
        }

        // Line
        ctx.strokeStyle = '#3b82f6';
        ctx.lineWidth = 2;
        ctx.beginPath();
        for (var i = 0; i < data.length; i++) {
            var x = padding + (graphW / (data.length - 1)) * i;
            var y = padding + graphH - (data[i] / max) * graphH;
            if (i === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
        }
        ctx.stroke();

        ctx.lineTo(padding + graphW, padding + graphH);
        ctx.lineTo(padding, padding + graphH);
        ctx.closePath();
        ctx.fillStyle = 'rgba(59, 130, 246, 0.1)';
        ctx.fill();
    },

    /**
     * Show a toast notification.
     */
    toast: function(message, type) {
        type = type || 'info';
        var container = document.getElementById('toast-container');
        var el = document.createElement('div');
        el.className = 'toast ' + type;
        el.textContent = message;
        container.appendChild(el);
        setTimeout(function() {
            el.style.opacity = '0';
            setTimeout(function() { el.remove(); }, 300);
        }, 3000);
    }
};

/**
 * Escape HTML special characters to prevent XSS.
 */
function escapeHtml(str) {
    if (!str) return '';
    return str.replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;');
}
