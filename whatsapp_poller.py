#!/usr/bin/env python3
"""
WhatsApp Message Poller for AMOR
Smart polling - only checks when bridge is connected
"""

import sqlite3
import time
import os
import json
import requests

DB_PATH = "/home/fuckall/whatsapp-bridge/store/messages.db"
QUEUE_FILE = "/tmp/whatsapp_in.txt"
LAST_MSG_FILE = "/tmp/whatsapp_last_msg.json"
USER_MAP_PATH = "/home/fuckall/amorshi/users/user_map.json"
BRIDGE_URL = "http://localhost:8080/api/status"


def check_bridge_connected():
    """Check if WhatsApp bridge is connected - try sending test message"""
    try:
        resp = requests.post(
            "http://localhost:8080/api/send",
            json={"recipient": "test", "message": "ping"},
            timeout=2,
        )
        # If it returns, bridge is up (even if test fails)
        return True
    except:
        return False


def restart_bridge():
    """Restart WhatsApp bridge if disconnected"""
    os.system("sudo systemctl restart whatsapp-bridge")
    print("🔄 Restarted WhatsApp bridge")


def load_user_map():
    """Load user map to get allowed WhatsApp JIDs"""
    try:
        with open(USER_MAP_PATH, "r") as f:
            data = json.load(f)
        users = data.get("users", {})
        allowed_jids = {}
        for name, info in users.items():
            if "whatsapp" in info:
                allowed_jids[info["whatsapp"].lower()] = name
        return allowed_jids
    except Exception as e:
        print(f"Error loading user map: {e}")
        return {}


def get_last_check_time():
    """Get the timestamp of last checked message"""
    if os.path.exists(LAST_MSG_FILE):
        with open(LAST_MSG_FILE, "r") as f:
            return json.load(f)
    return {"timestamp": 0, "jid": None}


def save_last_check(timestamp, jid):
    """Save last checked message info"""
    with open(LAST_MSG_FILE, "w") as f:
        json.dump({"timestamp": timestamp, "jid": jid}, f)


def check_new_messages(allowed_jids):
    """Check for new messages only from allowed users"""
    last_check = get_last_check_time()

    # Normalize JIDs for comparison (strip @lid, @s.whatsapp.net etc)
    def normalize_jid(jid):
        return jid.split("@")[0].lower()

    allowed_normalized = {normalize_jid(j): n for j, n in allowed_jids.items()}

    try:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        cursor.execute("""
            SELECT m.content, m.sender, m.timestamp, c.name, m.chat_jid
            FROM messages m
            JOIN chats c ON m.chat_jid = c.jid
            WHERE m.sender != 'me'
            AND m.content IS NOT NULL
            AND m.content != ''
            ORDER BY m.timestamp DESC
            LIMIT 20
        """)

        rows = cursor.fetchall()
        conn.close()

        for row in rows:
            content = row[0].strip()
            sender = row[1].lower()
            timestamp = str(row[2])
            name = row[3]
            chat_jid = row[4]

            if not content:
                continue

            sender_normalized = normalize_jid(sender)

            if (
                last_check.get("jid") == sender
                and last_check.get("timestamp") == timestamp
            ):
                continue

            if sender_normalized in allowed_normalized:
                save_last_check(timestamp, sender)
                sender_name = allowed_normalized[sender_normalized]
                return {
                    "jid": chat_jid,
                    "sender": sender,
                    "sender_name": sender_name,
                    "content": content,
                    "name": name,
                    "timestamp": timestamp,
                }

        return None

    except Exception as e:
        print(f"Error checking messages: {e}")
        return None


def write_to_queue(msg):
    """Write message to AMOR input queue"""
    # Format: WHATSAPP|JID|NAME|MESSAGE
    line = f"WHATSAPP|{msg['jid']}|{msg['sender_name']}|{msg['content']}\n"
    with open(QUEUE_FILE, "a") as f:
        f.write(line)
    print(f"📩 WhatsApp from {msg['sender_name']}: {msg['content'][:50]}...")


def main():
    print("🤖 WhatsApp poller starting...", flush=True)
    print("   Smart polling - checks bridge status first", flush=True)

    allowed_jids = load_user_map()
    print(f"   Allowed JIDs: {list(allowed_jids.keys())}", flush=True)

    consecutive_failures = 0

    while True:
        # Check if bridge is connected
        if not check_bridge_connected():
            print("⚠️ Bridge disconnected, restarting...", flush=True)
            restart_bridge()
            consecutive_failures += 1
            time.sleep(5)
            continue

        # Check for new messages
        msg = check_new_messages(allowed_jids)
        if msg:
            write_to_queue(msg)
            print(f"✅ Queued message from {msg['sender_name']}", flush=True)
            consecutive_failures = 0
        else:
            consecutive_failures += 1

        # If lots of failures, might be bridge issue
        if consecutive_failures > 10:
            print("⚠️ Many failures, checking bridge...", flush=True)
            if not check_bridge_connected():
                restart_bridge()
            consecutive_failures = 0

        time.sleep(3)


if __name__ == "__main__":
    main()
