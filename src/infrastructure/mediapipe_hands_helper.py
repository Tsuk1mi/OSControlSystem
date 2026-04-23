import json
import struct
import sys


def emit(payload):
    sys.stdout.write(json.dumps(payload, separators=(",", ":")) + "\n")
    sys.stdout.flush()


def read_exact(size):
    data = sys.stdin.buffer.read(size)
    if not data:
        return b""
    while len(data) < size:
        chunk = sys.stdin.buffer.read(size - len(data))
        if not chunk:
            return b""
        data += chunk
    return data


def main():
    try:
        import mediapipe as mp
        import numpy as np
    except Exception as exc:
        emit(
            {
                "ready": False,
                "error": (
                    "Пакет mediapipe не установлен для этого Python (%s). "
                    "Установите: py -3 -m pip install mediapipe"
                )
                % (exc,),
            }
        )
        return 1

    try:
        hands = mp.solutions.hands.Hands(
            static_image_mode=False,
            max_num_hands=1,
            model_complexity=0,
            min_detection_confidence=0.55,
            min_tracking_confidence=0.5,
        )
    except Exception as exc:
        emit({"ready": False, "error": "Failed to initialize MediaPipe Hands: %s" % exc})
        return 1

    emit(
        {
            "ready": True,
            "backend": "MediaPipe Hands (python)",
            "version": getattr(mp, "__version__", "unknown"),
        }
    )

    while True:
        header = read_exact(12)
        if not header:
            break

        width, height, payload_size = struct.unpack("<III", header)
        payload = read_exact(payload_size)
        if not payload:
            emit({"ok": False, "error": "Unexpected EOF while reading frame payload"})
            break

        try:
            frame = np.frombuffer(payload, dtype=np.uint8).reshape((height, width, 3))
            result = hands.process(frame)
            if not result.multi_hand_landmarks:
                emit({"ok": True, "landmarks": [], "handedness": None})
                continue

            hand_landmarks = result.multi_hand_landmarks[0]
            points = [[lm.x, lm.y, lm.z] for lm in hand_landmarks.landmark]
            handedness = None
            if result.multi_handedness:
                cls = result.multi_handedness[0].classification
                if cls:
                    handedness = cls[0].label
            emit({"ok": True, "landmarks": points, "handedness": handedness})
        except Exception as exc:
            emit({"ok": False, "error": "Frame processing failed: %s" % exc})

    try:
        hands.close()
    except Exception:
        pass
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
