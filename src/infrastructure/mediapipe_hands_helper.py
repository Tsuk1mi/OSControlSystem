import json
import os
import struct
import sys

# Официальная модель (float16) — скачивается один раз рядом со скриптом.
MODEL_URL = (
    "https://storage.googleapis.com/mediapipe-models/hand_landmarker/"
    "hand_landmarker/float16/1/hand_landmarker.task"
)


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


def model_file_path():
    base = os.path.dirname(os.path.abspath(__file__))
    return os.path.join(base, "hand_landmarker.task")


def ensure_task_model_file():
    path = model_file_path()
    if os.path.isfile(path) and os.path.getsize(path) > 1_000_000:
        return path, None
    try:
        import urllib.request

        tmp = path + ".part"
        sys.stderr.write("MediaPipe: загрузка модели hand_landmarker.task (один раз)...\n")
        sys.stderr.flush()
        with urllib.request.urlopen(MODEL_URL, timeout=120) as resp:
            with open(tmp, "wb") as out:
                while True:
                    chunk = resp.read(1 << 20)
                    if not chunk:
                        break
                    out.write(chunk)
        os.replace(tmp, path)
    except Exception as exc:
        return None, "Не удалось скачать модель: %s. URL: %s" % (exc, MODEL_URL)
    return path, None


def main():
    try:
        import numpy as np
    except Exception as exc:
        emit({"ready": False, "error": "numpy: %s" % exc})
        return 1

    try:
        import mediapipe as mp
    except Exception as exc:
        ver = sys.version.split()[0]
        exe = sys.executable
        emit(
            {
                "ready": False,
                "error": "mediapipe: %s. Python %s. %s — pip: %s -m pip install mediapipe numpy"
                % (exc, ver, exe, exe),
            }
        )
        return 1

    version = getattr(mp, "__version__", "unknown")
    hands_legacy = None
    landmarker_tasks = None
    legacy_init_error = None

    if getattr(mp, "solutions", None) and getattr(mp.solutions, "hands", None):
        try:
            hands_legacy = mp.solutions.hands.Hands(
                static_image_mode=False,
                max_num_hands=1,
                model_complexity=1,
                min_detection_confidence=0.45,
                min_tracking_confidence=0.40,
            )
        except Exception as exc:
            legacy_init_error = exc
            hands_legacy = None

    if hands_legacy is None:
        mpath, merr = ensure_task_model_file()
        if merr:
            err = merr
            if legacy_init_error:
                err = "%s; legacy Hands: %s" % (merr, legacy_init_error)
            emit({"ready": False, "error": err})
            return 1
        try:
            BaseOptions = mp.tasks.BaseOptions
            HandLandmarker = mp.tasks.vision.HandLandmarker
            HandLandmarkerOptions = mp.tasks.vision.HandLandmarkerOptions
            RunningMode = mp.tasks.vision.RunningMode
            options = HandLandmarkerOptions(
                base_options=BaseOptions(model_asset_path=mpath),
                running_mode=RunningMode.VIDEO,
                num_hands=1,
                min_hand_detection_confidence=0.45,
                min_hand_presence_confidence=0.40,
                min_tracking_confidence=0.40,
            )
            landmarker_tasks = HandLandmarker.create_from_options(options)
        except Exception as exc:
            msg = "HandLandmarker (tasks) %s" % exc
            if legacy_init_error:
                msg = "%s; legacy: %s" % (msg, legacy_init_error)
            emit({"ready": False, "error": msg})
            return 1
        api_label = "MediaPipe HandLandmarker (tasks) %s" % version
    else:
        api_label = "MediaPipe Hands (solutions) %s" % version

    emit(
        {
            "ready": True,
            "backend": api_label,
            "version": version,
        }
    )

    ts_ms = 0
    try:
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
            except Exception as exc:
                emit({"ok": False, "error": "Bad frame: %s" % exc})
                continue

            try:
                if hands_legacy is not None:
                    result = hands_legacy.process(frame)
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
                else:
                    ts_ms += 33
                    mp_image = mp.Image(image_format=mp.ImageFormat.SRGB, data=frame)
                    tr = landmarker_tasks.detect_for_video(mp_image, ts_ms)
                    if not tr.hand_landmarks:
                        emit({"ok": True, "landmarks": [], "handedness": None})
                        continue
                    h = tr.hand_landmarks[0]
                    points = [[float(lm.x), float(lm.y), float(lm.z)] for lm in h]
                    handedness = None
                    if tr.handedness and tr.handedness[0]:
                        handedness = tr.handedness[0][0].category_name
                    emit({"ok": True, "landmarks": points, "handedness": handedness})
            except Exception as exc:
                emit({"ok": False, "error": "Frame: %s" % exc})
    finally:
        try:
            if hands_legacy is not None:
                hands_legacy.close()
        except Exception:
            pass
        try:
            if landmarker_tasks is not None:
                landmarker_tasks.close()
        except Exception:
            pass
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
