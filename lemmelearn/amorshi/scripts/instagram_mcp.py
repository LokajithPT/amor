#!/usr/bin/env python3
"""
Instagram MCP Server - exposes Instagram DM tools via Model Context Protocol
Usage: python instagram_mcp.py
"""

import os
import json
import sys
from datetime import datetime

from mcp.server.fastmcp import FastMCP

SESSION_FILE = "/home/fuckall/instagram_session.json"
mcp = FastMCP("Instagram MCP", json_response=True)


def load_client():
    """Load or validate Instagram client session"""
    from instagrapi import Client

    cl = Client()

    if os.path.exists(SESSION_FILE):
        cl.load_settings(SESSION_FILE)

    try:
        cl.get_timeline_feed()
        return cl
    except Exception as e:
        print(f"Session invalid: {e}", file=sys.stderr)
        return None


def save_session(cl):
    """Persist session for reuse"""
    cl.dump_settings(SESSION_FILE)


@mcp.tool()
def instagram_send_dm(user_id: str, message: str) -> str:
    """Send a DM to a user on Instagram"""
    cl = load_client()
    if not cl:
        return json.dumps({"error": "Not logged in. Run login first."})

    try:
        cl.direct_send(message, user_ids=[user_id])
        return json.dumps({"success": True, "message": f"DM sent to {user_id}"})
    except Exception as e:
        return json.dumps({"error": str(e)})


@mcp.tool()
def instagram_check_dms() -> str:
    """Check for unread Instagram DMs - returns list of new messages"""
    cl = load_client()
    if not cl:
        return json.dumps({"error": "Not logged in"})

    try:
        threads = cl.direct_threads()
        pending = []

        for thread in threads:
            if thread.unread_count > 0:
                messages = thread.messages
                for msg in messages[: thread.unread_count]:
                    if msg.is_sent_by_me == 0:
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

        return json.dumps({"dms": pending, "count": len(pending)})
    except Exception as e:
        return json.dumps({"error": str(e)})


@mcp.tool()
def instagram_list_threads(limit: int = 10) -> str:
    """List recent Instagram DM threads"""
    cl = load_client()
    if not cl:
        return json.dumps({"error": "Not logged in"})

    try:
        threads = cl.direct_threads()
        result = []

        for thread in threads[:limit]:
            result.append(
                {
                    "thread_id": thread.thread_id,
                    "username": thread.users[0].username if thread.users else "unknown",
                    "user_id": thread.users[0].pk if thread.users else None,
                    "unread": thread.unread_count,
                    "last_message": thread.messages[0].text if thread.messages else "",
                    "last_timestamp": thread.messages[0].timestamp.isoformat()
                    if thread.messages and thread.messages[0].timestamp
                    else "",
                }
            )

        return json.dumps({"threads": result})
    except Exception as e:
        return json.dumps({"error": str(e)})


@mcp.tool()
def instagram_read_thread(thread_id: str, limit: int = 20) -> str:
    """Read messages from a specific thread"""
    cl = load_client()
    if not cl:
        return json.dumps({"error": "Not logged in"})

    try:
        thread = cl.direct_thread(thread_id, amount=limit)
        messages = []

        for msg in thread.messages:
            messages.append(
                {
                    "text": msg.text,
                    "sender_id": msg.user_id,
                    "is_mine": msg.is_sent_by_me == 1,
                    "timestamp": msg.timestamp.isoformat() if msg.timestamp else "",
                }
            )

        return json.dumps({"thread_id": thread_id, "messages": messages})
    except Exception as e:
        return json.dumps({"error": str(e)})


@mcp.tool()
def instagram_get_user(username: str) -> str:
    """Get user info by username"""
    cl = load_client()
    if not cl:
        return json.dumps({"error": "Not logged in"})

    try:
        user = cl.user_info_by_username(username)
        return json.dumps(
            {
                "user_id": str(user.pk),
                "username": user.username,
                "full_name": user.full_name,
                "is_private": user.is_private,
                "followers": user.follower_count,
                "following": user.following_count,
            }
        )
    except Exception as e:
        return json.dumps({"error": str(e)})


@mcp.tool()
def instagram_login(username: str = "amor_be_chillin", password: str = "") -> str:
    """Login to Instagram - use only if session is invalid"""
    if not password:
        return json.dumps({"error": "Password required"})

    from instagrapi import Client

    cl = Client()
    try:
        cl.login(username, password)
        save_session(cl)
        return json.dumps({"success": True, "message": f"Logged in as {username}"})
    except Exception as e:
        return json.dumps({"error": str(e)})


@mcp.tool()
def instagram_status() -> str:
    """Check Instagram connection status"""
    cl = load_client()
    if cl:
        return json.dumps({"status": "connected", "username": cl.username})
    else:
        return json.dumps({"status": "disconnected"})


if __name__ == "__main__":
    print("Starting Instagram MCP Server...", file=sys.stderr)
    mcp.run()
