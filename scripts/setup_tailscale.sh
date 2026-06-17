#!/usr/bin/env bash
# Tailscale setup for ghostpsalm.com personal assistant server.
# Run once on the server. After this, the service is reachable from any
# Tailscale-connected device (phone, laptop, desktop) without port forwarding.
set -euo pipefail

SERVICE_PORT=3000
MACHINE_NAME="ghostpsalm-server"

echo "=== Installing Tailscale ==="
if command -v tailscale &>/dev/null; then
    echo "Tailscale already installed: $(tailscale --version | head -1)"
else
    curl -fsSL https://tailscale.com/install.sh | sh
fi

echo ""
echo "=== Authenticating with Tailscale ==="
echo "A browser URL will appear. Open it to log in, then return here."
sudo tailscale up --hostname="$MACHINE_NAME" --ssh

echo ""
echo "=== Fetching Tailscale IP ==="
TAILSCALE_IP=$(tailscale ip -4)
echo "Your Tailscale IPv4: $TAILSCALE_IP"

echo ""
echo "=== Optional: provision a Tailscale HTTPS certificate ==="
echo "Tailscale's MagicDNS provides a hostname like:"
echo "  ${MACHINE_NAME}.your-tailnet-name.ts.net"
echo ""
echo "To get a cert (run manually after DNS propagates):"
echo "  sudo tailscale cert ${MACHINE_NAME}.your-tailnet-name.ts.net"
echo "  # This writes: /var/lib/tailscale/certs/..."
echo ""
echo "Then update .env:"
echo "  BIND_ADDR=${TAILSCALE_IP}:${SERVICE_PORT}"
echo "  TLS_CERT_PATH=/var/lib/tailscale/certs/<hostname>.crt"
echo "  TLS_KEY_PATH=/var/lib/tailscale/certs/<hostname>.key"
echo ""
echo "=== Firewall: allow on Tailscale interface only ==="
echo "If using firewalld (Fedora/RHEL):"
echo "  sudo firewall-cmd --zone=trusted --add-interface=tailscale0 --permanent"
echo "  sudo firewall-cmd --reload"
echo ""
echo "=== Done ==="
echo "Update BIND_ADDR in .env to ${TAILSCALE_IP}:${SERVICE_PORT}"
echo "Restart the service and connect from any Tailscale device."
