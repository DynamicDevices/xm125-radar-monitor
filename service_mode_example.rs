// Example systemd service integration
// Create /etc/systemd/system/xm125-radar.service:

/*
[Unit]
Description=XM125 Radar Monitor Service
After=network.target

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/xm125-radar-monitor presence --continuous --fifo-output --quiet
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
*/

// Usage:
// sudo systemctl enable xm125-radar.service
// sudo systemctl start xm125-radar.service
// sudo systemctl status xm125-radar.service

// The service will:
// 1. Start automatically on boot
// 2. Write to /tmp/presence FIFO continuously
// 3. Restart automatically if it crashes
// 4. Log to systemd journal

// Read FIFO data:
// cat /tmp/presence
// tail -f /tmp/presence | your_application
