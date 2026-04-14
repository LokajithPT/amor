#!/usr/bin/env python3
"""Service status checker for AMOR"""

import subprocess
import json
import os


def check_service(name, pid_file=None, port=None, process_name=None):
    """Check if a service is running"""
    status = {"name": name, "running": False, "details": ""}

    # Check by PID file
    if pid_file and os.path.exists(pid_file):
        try:
            with open(pid_file) as f:
                pid = int(f.read().strip())
            if os.path.exists(f"/proc/{pid}"):
                status["running"] = True
                status["details"] = f"PID: {pid}"
                return status
        except:
            pass

    # Check by port
    if port:
        result = subprocess.run(["ss", "-tlnp"], capture_output=True, text=True)
        if f":{port}" in result.stdout:
            status["running"] = True
            status["details"] = f"Listening on port {port}"
            return status

    # Check by process name
    if process_name:
        result = subprocess.run(
            ["pgrep", "-f", process_name], capture_output=True, text=True
        )
        if result.returncode == 0 and result.stdout.strip():
            pids = result.stdout.strip().split("\n")
            status["running"] = True
            status["details"] = f"PIDs: {', '.join(pids)}"
            return status

    status["details"] = "Not running"
    return status


def main():
    services = []

    # Check WhatsApp bridge (Go process)
    services.append(check_service("WhatsApp Bridge", process_name="whatsapp-bridge"))

    # Check WhatsApp MCP server
    services.append(check_service("WhatsApp MCP", port=8000))

    # Check WhatsApp poller
    services.append(check_service("WhatsApp Poller", process_name="whatsapp_poller.py"))

    # Check AMOR bot
    services.append(check_service("AMOR Bot", pid_file="/tmp/amor.pid"))

    # Check Telegram bot token validity
    try:
        import requests

        config_path = "/home/fuckall/amorshi/shit.cfg"
        if os.path.exists(config_path):
            import json

            with open(config_path) as f:
                config = json.load(f)
            token = config.get("telegram_bot_token", "")
            if token and token != "REPLACE_WITH_TELEGRAM_BOT_TOKEN":
                result = requests.get(
                    f"https://api.telegram.org/bot{token}/getMe", timeout=5
                )
                if result.json().get("ok"):
                    services.append(
                        {
                            "name": "Telegram Bot API",
                            "running": True,
                            "details": "Token valid",
                        }
                    )
                else:
                    services.append(
                        {
                            "name": "Telegram Bot API",
                            "running": False,
                            "details": "Token invalid",
                        }
                    )
            else:
                services.append(
                    {
                        "name": "Telegram Bot API",
                        "running": False,
                        "details": "No token configured",
                    }
                )
        else:
            services.append(
                {
                    "name": "Telegram Bot API",
                    "running": False,
                    "details": "Config not found",
                }
            )
    except Exception as e:
        services.append(
            {"name": "Telegram Bot API", "running": False, "details": f"Error: {e}"}
        )

    # Print status
    print("=== AMOR Service Status ===")
    for s in services:
        icon = "✅" if s["running"] else "❌"
        print(f"{icon} {s['name']}: {s['details']}")

    print("\n=== JSON Output ===")
    print(json.dumps(services, indent=2))


if __name__ == "__main__":
    main()
