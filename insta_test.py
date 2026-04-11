#!/usr/bin/env python3
# Instagram DM tool for AMOR
import sys
from instagrapi import Client
import os

SESSION_FILE = "instagram_session.json"

cl = Client()

# Try to load existing session
if os.path.exists(SESSION_FILE):
    print("Loading session...")
    cl.load_settings(SESSION_FILE)

    # Check if still valid
    try:
        me = cl.account_info()
        print(f"Session valid! Logged in as {me.username}")
    except:
        print("Session expired, need to re-login...")
        USERNAME = "amor_be_chillin"
        PASSWORD = "idontlikethisgameanymore"
        cl.login(USERNAME, PASSWORD)
        cl.dump_settings(SESSION_FILE)
        print("New session saved!")

# Test sending a DM
print("\nTesting DM to lokaj1th...")
import time

time.sleep(2)

try:
    user_id = cl.user_id_from_username("lokaj1th")
    print(f"User ID: {user_id}")
    time.sleep(1)
    cl.direct_send("Hey! AMOR is online from your Pi 🔥", user_ids=[user_id])
    print("DM sent!")
except Exception as e:
    print(f"DM error: {e}")
