#!/usr/bin/env python3
# Instagram DM tool for AMOR
import sys
from instagrapi import Client
import json
import os

USERNAME = os.getenv("AMOR_IG_USERNAME", "amor_be_chillin")
PASSWORD = os.getenv("AMOR_IG_PASSWORD")

if not PASSWORD:
    raise RuntimeError("Set AMOR_IG_PASSWORD before running this script")

cl = Client()
print("Logging in...")
cl.login(USERNAME, PASSWORD)
print(f"Logged in as {USERNAME}")

# Save session
cl.dump_settings("instagram_session.json")
print("Session saved!")

# Test - get own profile
me = cl.account_info()
print(f"User ID: {me.pk}")
print(f"Username: {me.username}")
print(f"Followers: {me.follower_count}")
print("SUCCESS!")
