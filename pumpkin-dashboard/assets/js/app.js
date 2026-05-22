/**
 * Pumpkin Dashboard - Main Application Logic
 *
 * Handles routing, API calls, WebSocket connections, and page rendering.
 */

var App = {
    token: null,
    ws: null,
    refreshInterval: null,
    tpsHistory: [],
    memHistory: [],
    consoleLinesMax: 500,

    /**
     * Initialize the application.
     */
    init: function() {
        var saved = sessionStorage.getItem('dashboard_token');
        if (saved) {
            App.token = saved;
            App.showApp();
        }

        document.getElementById('login-form').addEventListener('submit', function(e) {
            e.preventDefault();
            App.login();
        });

        window.addEventListener('hashchange', function() {
            if (App.token) App.route();
        });
    },

    /**
     * Authenticate with the server.
     */
    login: function() {
        var password = document.getElementById('login-password').value;
        App.api('POST', '/auth/login', { password: password })
            .then(function(data) {
                App.token = data.token;
                sessionStorage.setItem('dashboard_token', data.token);
                document.getElementById('login-error').textContent = '';
                App.showApp();
            })
            .catch(function(err) {
                document.getElementById('login-error').textContent = err.message || 'Login failed';
            });
    },

    /**
     * Show the main application view and start routing.
     */
    showApp: function() {
        document.getElementById('login-view').classList.add('hidden');
        document.getElementById('app-view').classList.remove('hidden');
        App.route();
        App.connectWebSocket();
        App.startAutoRefresh();
    },

    /**
     * Hash-based router.
     */
    route: function() {
        var hash = window.location.hash || '#/';
        var page = hash.replace('#/', '') || 'overview';

        // Update active nav link
        var links = document.querySelectorAll('.nav-link');
        for (var i = 0; i < links.length; i++) {
            links[i].classList.remove('active');
            if (links[i].getAttribute('data-page') === page) {
                links[i].classList.add('active');
            }
        }

        switch (page) {
            case 'overview': App.renderOverview(); break;
            case 'players': App.renderPlayers(); break;
            case 'console': App.renderConsole(); break;
            case 'settings': App.renderSettings(); break;
            case 'performance': App.renderPerformance(); break;
            default: App.renderOverview();
        }
    },

    /**
     * Render the Overview page.
     */
    renderOverview: function() {
        var content = document.getElementById('page-content');
        content.innerHTML = '<div class="page-header"><h2>Server Overview</h2></div>' +
            '<div id="overview-stats" class="stats-grid"><p style="color:var(--text-muted)">Loading...</p></div>';

        App.api('GET', '/server/status').then(function(data) {
            var tpsColor = data.tps >= 18 ? 'success' : (data.tps >= 15 ? 'warning' : 'danger');
            var html = Components.statCard('Status', Components.statusBadge(data.online), '') +
                Components.statCard('Players', data.player_count + '/' + data.max_players, 'info') +
                Components.statCard('TPS', data.tps.toFixed(1), tpsColor) +
                Components.statCard('Avg Tick', data.avg_tick_ms.toFixed(1) + 'ms', '') +
                Components.statCard('Memory', data.memory_usage_mb + ' MB', '') +
                Components.statCard('Uptime', App.formatUptime(data.uptime_secs), '') +
                Components.statCard('Version', data.version, '') +
                Components.statCard('MOTD', escapeHtml(data.motd), '');
            document.getElementById('overview-stats').innerHTML = html;

            // Track for charts
            App.tpsHistory.push(data.tps);
            if (App.tpsHistory.length > 60) App.tpsHistory.shift();
            App.memHistory.push(data.memory_usage_mb);
            if (App.memHistory.length > 60) App.memHistory.shift();
        }).catch(function() {
            document.getElementById('overview-stats').innerHTML = '<p style="color:var(--danger)">Failed to load status</p>';
        });
    },

    /**
     * Render the Players page.
     */
    renderPlayers: function() {
        var content = document.getElementById('page-content');
        content.innerHTML = '<div class="page-header"><h2>Players</h2></div>' +
            '<div id="players-list"><p style="color:var(--text-muted)">Loading...</p></div>';

        App.api('GET', '/players').then(function(data) {
            document.getElementById('players-list').innerHTML = Components.playerTable(data);
        }).catch(function() {
            document.getElementById('players-list').innerHTML = '<p style="color:var(--danger)">Failed to load players</p>';
        });
    },

    /**
     * Render the Console page.
     */
    renderConsole: function() {
        var content = document.getElementById('page-content');
        content.innerHTML = '<div class="page-header"><h2>Console</h2></div>' +
            '<div class="console-container">' +
                '<div class="console-output" id="console-output"></div>' +
                '<div class="console-input">' +
                    '<input type="text" id="console-cmd" placeholder="Enter command..." />' +
                    '<button onclick="App.sendCommand()">Send</button>' +
                '</div>' +
            '</div>';

        document.getElementById('console-cmd').addEventListener('keydown', function(e) {
            if (e.key === 'Enter') App.sendCommand();
        });
    },

    /**
     * Render the Settings page.
     */
    renderSettings: function() {
        var content = document.getElementById('page-content');
        content.innerHTML = '<div class="page-header"><h2>Server Settings</h2></div>' +
            '<div id="settings-content" class="settings-grid"><p style="color:var(--text-muted)">Loading...</p></div>';

        App.api('GET', '/config').then(function(data) {
            var html = '';
            var entries = App.flattenObject(data);
            for (var key in entries) {
                html += '<div class="setting-item">' +
                    '<span class="setting-key">' + escapeHtml(key) + '</span>' +
                    '<span class="setting-value">' + escapeHtml(String(entries[key])) + '</span>' +
                    '</div>';
            }
            document.getElementById('settings-content').innerHTML = html || '<p style="color:var(--text-muted)">No settings available</p>';
        }).catch(function() {
            document.getElementById('settings-content').innerHTML = '<p style="color:var(--danger)">Failed to load settings</p>';
        });
    },

    /**
     * Render the Performance page.
     */
    renderPerformance: function() {
        var content = document.getElementById('page-content');
        content.innerHTML = '<div class="page-header"><h2>Performance</h2></div>' +
            '<div class="chart-container"><h3>TPS History</h3><canvas id="tps-chart" class="chart-canvas"></canvas></div>' +
            '<div class="chart-container"><h3>Memory Usage</h3><canvas id="mem-chart" class="chart-canvas"></canvas></div>';

        setTimeout(function() {
            Components.drawTpsChart('tps-chart', App.tpsHistory);
            Components.drawMemoryChart('mem-chart', App.memHistory);
        }, 50);
    },

    /**
     * Make an API call to the dashboard backend.
     */
    api: function(method, path, body) {
        var opts = {
            method: method,
            headers: { 'Content-Type': 'application/json' }
        };
        if (App.token) {
            opts.headers['Authorization'] = 'Bearer ' + App.token;
        }
        if (body) {
            opts.body = JSON.stringify(body);
        }
        return fetch('/api/v1' + path, opts).then(function(res) {
            if (res.status === 401) {
                App.logout();
                throw new Error('Session expired');
            }
            if (!res.ok) {
                return res.json().then(function(d) { throw new Error(d.error || 'Request failed'); });
            }
            return res.json();
        });
    },

    /**
     * Connect to the WebSocket for live console streaming.
     */
    connectWebSocket: function() {
        if (App.ws) App.ws.close();
        var proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        var url = proto + '//' + window.location.host + '/api/v1/ws';
        App.ws = new WebSocket(url);

        App.ws.onmessage = function(event) {
            App.appendConsole(event.data);
        };

        App.ws.onclose = function() {
            setTimeout(function() {
                if (App.token) App.connectWebSocket();
            }, 5000);
        };
    },

    /**
     * Append a line to the console output.
     */
    appendConsole: function(line) {
        var output = document.getElementById('console-output');
        if (!output) return;

        var el = document.createElement('div');
        el.className = 'log-line';
        if (line.indexOf('[ERROR]') !== -1) el.className += ' error';
        else if (line.indexOf('[WARN]') !== -1) el.className += ' warn';
        else el.className += ' info';
        el.textContent = line;
        output.appendChild(el);

        // Limit lines
        while (output.children.length > App.consoleLinesMax) {
            output.removeChild(output.firstChild);
        }

        output.scrollTop = output.scrollHeight;
    },

    /**
     * Send a command from the console input.
     */
    sendCommand: function() {
        var input = document.getElementById('console-cmd');
        if (!input || !input.value.trim()) return;

        var cmd = input.value.trim();
        input.value = '';

        if (App.ws && App.ws.readyState === WebSocket.OPEN) {
            App.ws.send(cmd);
        } else {
            App.api('POST', '/console/command', { command: cmd }).then(function() {
                Components.toast('Command sent', 'success');
            }).catch(function(err) {
                Components.toast('Error: ' + err.message, 'error');
            });
        }
    },

    /**
     * Kick a player by UUID.
     */
    kickPlayer: function(uuid) {
        App.api('POST', '/players/' + uuid + '/kick', { reason: 'Kicked by dashboard' })
            .then(function() {
                Components.toast('Player kicked', 'success');
                App.renderPlayers();
            })
            .catch(function(err) {
                Components.toast('Error: ' + err.message, 'error');
            });
    },

    /**
     * Ban a player by UUID.
     */
    banPlayer: function(uuid) {
        if (!confirm('Are you sure you want to ban this player?')) return;
        App.api('POST', '/players/' + uuid + '/ban', { reason: 'Banned by dashboard' })
            .then(function() {
                Components.toast('Player banned', 'success');
                App.renderPlayers();
            })
            .catch(function(err) {
                Components.toast('Error: ' + err.message, 'error');
            });
    },

    /**
     * Log out and return to the login view.
     */
    logout: function() {
        App.token = null;
        sessionStorage.removeItem('dashboard_token');
        if (App.ws) App.ws.close();
        if (App.refreshInterval) clearInterval(App.refreshInterval);
        document.getElementById('login-view').classList.remove('hidden');
        document.getElementById('app-view').classList.add('hidden');
    },

    /**
     * Start auto-refresh of stats every 5 seconds.
     */
    startAutoRefresh: function() {
        if (App.refreshInterval) clearInterval(App.refreshInterval);
        App.refreshInterval = setInterval(function() {
            var hash = window.location.hash || '#/';
            var page = hash.replace('#/', '') || 'overview';
            if (page === 'overview') App.renderOverview();
            else if (page === 'performance') App.renderPerformance();
        }, 5000);
    },

    /**
     * Format seconds into a human-readable uptime string.
     */
    formatUptime: function(secs) {
        var d = Math.floor(secs / 86400);
        var h = Math.floor((secs % 86400) / 3600);
        var m = Math.floor((secs % 3600) / 60);
        if (d > 0) return d + 'd ' + h + 'h ' + m + 'm';
        if (h > 0) return h + 'h ' + m + 'm';
        return m + 'm';
    },

    /**
     * Flatten a nested object into key paths for display.
     */
    flattenObject: function(obj, prefix) {
        prefix = prefix || '';
        var result = {};
        for (var key in obj) {
            if (!obj.hasOwnProperty(key)) continue;
            var fullKey = prefix ? prefix + '.' + key : key;
            if (typeof obj[key] === 'object' && obj[key] !== null && !Array.isArray(obj[key])) {
                var nested = App.flattenObject(obj[key], fullKey);
                for (var nk in nested) {
                    result[nk] = nested[nk];
                }
            } else {
                result[fullKey] = obj[key];
            }
        }
        return result;
    }
};

// Initialize on DOM ready
document.addEventListener('DOMContentLoaded', App.init);
