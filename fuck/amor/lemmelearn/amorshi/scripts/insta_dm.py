#!/usr/bin/env python3
# Instagram DM tool for AMOR
import sys
import os
from instagrapi import Client

SESSION_FILE = "/home/fuckall/instagram_session.json"


def send_dm(recipient_username, message):
    cl = Client()

    # Load session if exists
    if os.path.exists(SESSION_FILE):
        try:
            cl.load_settings(SESSION_FILE)
        except:
            pass

    # Login if needed
    if not cl.user_id:
        USERNAME = os.getenv("AMOR_IG_USERNAME", "amor_be_chillin")
        PASSWORD = os.getenv("AMOR_IG_PASSWORD")
        if not PASSWORD:
            return "❌ Error: missing AMOR_IG_PASSWORD env var"
        cl.login(USERNAME, PASSWORD)
        cl.dump_settings(SESSION_FILE)

    # Find user ID by username
    try:
        user_id = cl.user_id_from_username(recipient_username)
        cl.send_direct_message(user_id, message)
        return f"✅ Sent to {recipient_username}: {message}"
    except Exception as e:
        return f"❌ Error: {e}"


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: insta_dm.py <username> <message>")
        sys.exit(1)

    result = send_dm(sys.argv[1], sys.argv[2])
    print(result)
