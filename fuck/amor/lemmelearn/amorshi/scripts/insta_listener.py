#!/usr/bin/env python3
# Instagram DM listener for AMOR - runs as daemon, checks for new messages
import os
import time
import json
import sys
from datetime import datetime

SESSION_FILE = "/home/fuckall/instagram_session.json"
CACHE_FILE = "/tmp/ig_threads_cache.json"


def load_client():
    from instagrapi import Client

    cl = Client()

    # Try to load session
    if os.path.exists(SESSION_FILE):
        cl.load_settings(SESSION_FILE)

    # Check if valid
    try:
        cl.get_timeline_feed()
        print(f"✅ Session valid, logged in as {cl.username}")
        return cl
    except:
        print("❌ Session invalid, need new login")
        return None


def save_session(cl):
    cl.dump_settings(SESSION_FILE)
    print("💾 Session saved")


def get_pending_dms(cl):
    try:
        threads = cl.direct_threads()
        pending = []
        for thread in threads:
            if thread.unread_count > 0:
                messages = thread.messages
                # Get only new unread messages
                for msg in messages[: thread.unread_count]:
                    if msg.is_sent_by_me == 0:  # Not from me
                        pending.append(
                            {
                                "thread_id": thread.thread_id,
                                "user_id": thread.users[0].pk if thread.users else None,
                                "username": thread.users[0].username
                                if thread.users
                                else "unknown",
                                "text": msg.text,
                                "timestamp": msg.timestamp.isoformat()
                                if msg.timestamp
                                else "",
                            }
                        )
        return pending
    except Exception as e:
        print(f"Error getting DMs: {e}")
        return []


def send_dm(cl, user_id, text):
    try:
        cl.direct_send(text, user_ids=[user_id])
        return True
    except Exception as e:
        print(f"DM error: {e}")
        return False


if __name__ == "__main__":
    # Check for commands
    if len(sys.argv) > 1:
        command = sys.argv[1]

        if command == "send":
            # Usage: insta_listener.py send <user_id> <message>
            if len(sys.argv) >= 4:
                user_id = sys.argv[2]
                message = " ".join(sys.argv[3:])
                cl = load_client()
                if cl:
                    if send_dm(cl, user_id, message):
                        print("✅ DM sent!")
                    else:
                        print("❌ DM failed")

        elif command == "check":
            # Check for new messages and output as JSON
            cl = load_client()
            if cl:
                dms = get_pending_dms(cl)
                print(json.dumps(dms))

        elif command == "login":
            # Force new login
            from instagrapi import Client

            cl = Client()
            USERNAME = "amor_be_chillin"
            PASSWORD = "idontlikethisgameanymore"
            try:
                cl.login(USERNAME, PASSWORD)
                save_session(cl)
                print("✅ Login successful!")
            except Exception as e:
                print(f"❌ Login failed: {e}")

        else:
            print("Unknown command")

    else:
        print("Usage: insta_listener.py [send|check|login]")
