#!/home/fuckall/tars_voice/venv/bin/python3
import sys
import sounddevice as sd
from kokoro_onnx import Kokoro
import threading
import time

k = Kokoro("/home/fuckall/kokoro-v1.0.int8.onnx", "/home/fuckall/voices-v1.0.bin")

text = " ".join(sys.argv[1:])
text = text.replace("AMOR", "A M O R")


def play_audio():
    samples, sr = k.create(text, voice="am_michael", speed=1.0, lang="en-us")
    sd.play(samples, sr)
    # Wait with timeout
    for _ in range(30):  # max 30 seconds
        if sd.get_stream().active == False:
            break
        time.sleep(1)


# Run in background thread
t = threading.Thread(target=play_audio, daemon=True)
t.start()

# Exit quickly
sys.exit(0)
