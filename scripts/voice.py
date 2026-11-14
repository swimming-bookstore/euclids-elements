#!/usr/bin/env python3
"""Narrate a proposition in the lessac voice, on the measured highlight clock.

Two libraries keep the voice the same from video to video:

* assets/voice/letters/ — plate names (AB, C, CDEB, …), never synthesized as
  English words.
* assets/voice/words/ — function words (the, and, of, …). One clip is reused
  everywhere, so "the" does not change colour from sentence to sentence.

A missing clip is synthesized once, written into the library, then reused.
"""
import array
import os
import subprocess
import wave
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LETTERS = REPO / "assets" / "voice" / "letters"
WORDS = REPO / "assets" / "voice" / "words"
ROOT = Path("/tmp/elements")
TOKENS = ROOT / "ii3-speak.tsv"
CHANGES = ROOT / "changes.txt"
HIGHLIGHTS = ROOT / "highlights.txt"
WORD = ROOT / "ii3-words"
WORD.mkdir(exist_ok=True)
PIPER = "/tmp/piper/piper/piper"
MODEL = "/tmp/piper/voices/en_US-lessac-medium.onnx"
ESPEAK = "/tmp/piper/piper/espeak-ng-data"
ENV = {**os.environ, "LD_LIBRARY_PATH": "/tmp/piper/piper"}
SR = 22050
VIDEO = 190.54


def speakable(text: str) -> str:
    raw = "".join(c for c in text if c.isalpha())
    # Plate letters are names, not words. A semicolon is the fallback spelling
    # for a single letter. Pairs are not sent here: Piper reads "A; B." as "AC".
    # A, AC, CDEB, ED, AF, BE, AE, AD, and CE are exceptions. The stored A has lost its
    # glide, so "A." is heard as "I"; spelling it "A." again keeps the letter.
    # Joining that A onto C says "I, C", and "A; C." is the spelling that keeps
    # both. Joining the stored E onto CDEB says "C, D, B, B" — that E is heard
    # as B — and "C; D; E; B." is the spelling that keeps all four. That
    # spelling carries a high-pitched hiss the other words do not, so the
    # stored clip is the phrase with the high band shelved down. That E also
    # ends in a high chirp the shelf does not remove, so the stored clip fades
    # the tail out before the chirp. The same E makes ED "B, D"; "E. D." keeps
    # both letters, and that clip is shelved and faded the same way. The same
    # A makes AF "I, Earth". "A. F." keeps the letters, but that F is a
    # click, so the stored clip is "A; F." with the dead pause closed.
    # The same E makes BE "B, B"; "B. E." keeps both letters.
    # Joining the stored A and E says "I, B". "A. E." keeps both letters.
    # Joining the stored A and D says "I, D". "A. D." keeps both letters.
    # Joining the stored C and E says "C, B". "C. E." keeps both letters.
    if raw == "A":
        return "A."
    if raw == "AC":
        return "A; C."
    if raw == "AD":
        return "A. D."
    if raw == "AE":
        return "A. E."
    if raw == "CE":
        return "C. E."
    if raw == "CDEB":
        return "C; D; E; B."
    if raw == "ED":
        return "E. D."
    if raw == "AF":
        return "A; F."
    if raw == "BE":
        return "B. E."
    if raw and raw.isupper() and 1 <= len(raw) <= 4 and raw != "I":
        return "; ".join(raw) + "."
    spoken = text.replace("(", " ").replace(")", " ")
    spoken = spoken.strip(" ,.;:!?")
    spoken = " ".join(spoken.split())
    if spoken == "rectangle s":
        return "rectangles"
    return spoken


def load_tokens():
    out = []
    for line in TOKENS.read_text().splitlines():
        if not line.strip() or not line[0].isdigit():
            continue
        _start, dur, text = line.split("\t", 2)
        out.append((int(dur) / 1000.0, text))
    return out


def load_highlights(n: int):
    """One measured highlight start per spoken word. Preferred over the
    change-log match, which treats a paint flicker as a word.
    """
    if not HIGHLIGHTS.exists():
        return None
    times = []
    for line in HIGHLIGHTS.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        times.append(float(line.split()[0]))
    if len(times) != n:
        print(f"highlight count {len(times)} != {n}, using change log")
        return None
    for i in range(1, len(times)):
        if times[i] <= times[i - 1]:
            raise SystemExit(f"highlight not increasing at {i}")
    return times


def load_changes():
    return [float(l.split("\t")[0]) for l in CHANGES.read_text().splitlines() if l.strip()]


def visual_times(tokens, changes):
    long_toks = [i for i, t in enumerate(tokens) if t[0] >= 1.5]
    long_chs = [i for i in range(len(changes) - 1) if changes[i + 1] - changes[i] > 1.4]
    if len(long_toks) != len(long_chs):
        raise SystemExit(f"anchor mismatch {len(long_toks)} vs {len(long_chs)}")
    vis = [None] * len(tokens)
    for a, b in zip(long_toks, long_chs):
        vis[a] = changes[b]

    def fill(i0, i1, c0, c1):
        need = [i for i in range(i0, i1) if vis[i] is None]
        avail = list(range(c0, c1))
        while len(avail) > len(need) and len(avail) > 1:
            gaps = []
            for k in range(len(avail)):
                prev = changes[avail[k - 1]] if k else changes[max(0, avail[0] - 1)]
                gaps.append((changes[avail[k]] - prev, k))
            gaps.sort()
            avail.pop(gaps[0][1])
        if len(avail) == len(need) and need:
            for i, c in zip(need, avail):
                vis[i] = changes[c]
        elif need:
            t0 = vis[i0 - 1] if i0 else changes[0]
            t1 = vis[i1] if i1 < len(vis) and vis[i1] is not None else changes[-1]
            for k, i in enumerate(need):
                vis[i] = t0 + (t1 - t0) * (k + 1) / (len(need) + 1)

    fill(0, long_toks[0], 0, long_chs[0])
    for k in range(len(long_toks) - 1):
        fill(long_toks[k] + 1, long_toks[k + 1], long_chs[k] + 1, long_chs[k + 1])
    fill(long_toks[-1] + 1, len(tokens), long_chs[-1] + 1, len(changes))
    if any(v is None for v in vis):
        raise SystemExit("unplaced word")
    for i in range(1, len(vis)):
        if vis[i] <= vis[i - 1]:
            raise SystemExit(f"non-monotonic {i} {vis[i-1]} {vis[i]}")
    return vis


def read_wav(path: Path):
    with wave.open(str(path)) as w:
        if w.getframerate() != SR or w.getnchannels() != 1:
            raise SystemExit(f"bad wav {path} {w.getframerate()} {w.getnchannels()}")
        s = array.array("h")
        s.frombytes(w.readframes(w.getnframes()))
        return s


def write_wav(path: Path, samples: array.array):
    with wave.open(str(path), "w") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes(samples.tobytes())


def trim(samples: array.array, thr=500):
    if not samples:
        return samples
    n = len(samples)
    a = 0
    while a < n and abs(samples[a]) < thr:
        a += 1
    b = n - 1
    while b > a and abs(samples[b]) < thr:
        b -= 1
    # keep a few ms of breath
    a = max(0, a - int(0.012 * SR))
    b = min(n - 1, b + int(0.04 * SR))
    return samples[a : b + 1]


def synth_one(job):
    i, text = job
    speech = speakable(text)
    raw = WORD / f"{i:03d}-raw.wav"
    subprocess.run(
        [PIPER, "-m", MODEL, "--espeak_data", ESPEAK, "-f", str(raw), "--length_scale", "0.92"],
        input=speech + "\n",
        text=True,
        env=ENV,
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    samples = trim(read_wav(raw))
    write_wav(WORD / f"{i:03d}.wav", samples)
    return i, speech, len(samples) / SR


def atempo_chain(speed: float) -> str:
    parts = []
    s = speed
    while s > 2.0:
        parts.append("atempo=2.0")
        s /= 2.0
    while s < 0.5:
        parts.append("atempo=0.5")
        s /= 0.5
    parts.append(f"atempo={s:.5f}")
    return ",".join(parts)


def plate_letters(text: str) -> str | None:
    raw = "".join(c for c in text if c.isalpha())
    if raw and raw.isupper() and 1 <= len(raw) <= 4 and raw != "I":
        return raw
    return None


def word_key(text: str) -> str | None:
    """Library key for a function word. Plate letters are not words.

    The stored "square" is not the word Piper says for "square": that clip
    drops the S and is heard as "where". The library clip is "skware",
    which keeps the leading S. Fading that S makes the word "where" again.
    The stored "a" is
    not the particle either: an isolated "a" is the letter name, and that
    clip is heard as "ah". The library clip is the schwa cut from
    "a straight line".
    """
    if plate_letters(text):
        return None
    key = "".join(c.lower() for c in text if c.isalpha() or c == "'")
    if key and (WORDS / f"{key}.wav").exists():
        return key
    return None


def library_clip(name: str) -> array.array | None:
    path = LETTERS / f"{name}.wav"
    if not path.exists():
        return None
    return read_wav(path)


def compose_plate(name: str) -> array.array | None:
    """Join the stored single-letter clips.

    Piper reads "A; B." as "AC", so a plate name is never synthesized as a
    phrase. Each letter is its own library clip, with a short breath between.
    A is not taken from the old single-letter clip: that clip has lost its
    glide and is heard as "I". speakable spells the name "A." instead, and
    that clip is what the library keeps. AC is not joined from the old A:
    "I, C". speakable spells that name "A; C." instead. CDEB is not joined
    either: the stored E is heard as B, so C plus D plus E plus B says
    "C, D, B, B". speakable spells that name "C; D; E; B." instead. That
    spelling carries a high-pitched hiss the other words do not, so the
    stored clip is the phrase with the high band shelved down. That E also
    ends in a high chirp the shelf does not remove, so the stored clip fades
    the tail out before the chirp. ED is not joined for the same E: E plus D
    says "B, D", and speakable spells that name "E. D." instead, shelved and
    faded the same way. AF is not joined for the same A: A plus F says
    "I, Earth". speakable spells that name "A; F." instead. "A. F." keeps
    the letters, but that F is a click, so the stored clip is "A; F." with
    the dead pause between the letters closed. BE is not
    joined for the same E: B plus E says "B, B", and speakable spells that
    name "B. E." instead. AE is not joined either: A plus E says "I, B",
    and speakable spells that name "A. E." instead. AD is not joined
    either: A plus D says "I, D", and speakable spells that name "A. D."
    instead. CE is not joined either: C plus E says "C, B", and speakable
    spells that name "C. E." instead.
    """
    if name in ("A", "AC", "CDEB", "ED", "AF", "BE", "AE", "AD", "CE"):
        return None
    clips = [library_clip(ch) for ch in name]
    if any(clip is None for clip in clips):
        return None
    gap = array.array("h", [0]) * int(0.06 * SR)
    out = array.array("h")
    for i, clip in enumerate(clips):
        if i:
            out.extend(gap)
        out.extend(clip)
    return out


def clip_for(i, text):
    """The stored clip for this token. Never time-stretched: speeding a
    letter-name turns C into D and swallows the consonant of a short word.
    """
    name = plate_letters(text)
    cached = library_clip(name) if name else None
    if cached is not None:
        return cached
    if name and len(name) > 1:
        composed = compose_plate(name)
        if composed is not None:
            LETTERS.mkdir(parents=True, exist_ok=True)
            write_wav(LETTERS / f"{name}.wav", composed)
            print("library compose", name)
            return composed
    common = word_key(text)
    if common:
        return read_wav(WORDS / f"{common}.wav")
    src = WORD / f"{i:03d}.wav"
    samples = read_wav(src)
    if name:
        # First time this name is spoken: keep the clip for later videos.
        LETTERS.mkdir(parents=True, exist_ok=True)
        write_wav(LETTERS / f"{name}.wav", samples)
        print("library add", name)
    return samples


def schedule(vis, durations, breath=0.04):
    """Start each clip when its word lights up.

    A clip is never started early: early speech is a word ahead of the figure.
    If the previous clip is still going, this one waits until it ends. Two
    words in the same breath is an overlap, not a faster highlight.
    """
    starts = []
    prev_end = 0.0
    for i, dur in enumerate(durations):
        start = max(vis[i], prev_end + (breath if i else 0.0))
        start = max(0.05, start)
        starts.append(start)
        prev_end = start + dur
    return starts


def main():
    tokens = load_tokens()
    highlights = load_highlights(len(tokens))
    if highlights is not None:
        vis = highlights
        print("highlights", len(vis))
    else:
        changes = load_changes()
        vis = visual_times(tokens, changes)
    jobs = [
        (i, text)
        for i, (_dur, text) in enumerate(tokens)
        if plate_letters(text) is None and word_key(text) is None
    ]
    print("synth", len(jobs), "library", len(tokens) - len(jobs))
    with ThreadPoolExecutor(max_workers=4) as pool:
        rows = list(pool.map(synth_one, jobs))
    rows.sort()
    for i, speech, dur in rows:
        if i < 8 or "of" == speech or speech.endswith(".") and ";" in speech:
            print(f"  {i:03d} {tokens[i][1]!r:18} -> {speech!r:18} {dur:.2f}s at {vis[i]:.2f}")
    clips = [clip_for(i, text) for i, (_dur, text) in enumerate(tokens)]
    durations = [len(s) / SR for s in clips]
    starts = schedule(vis, durations)
    late = max(starts[i] - vis[i] for i in range(len(vis)))
    early = max(vis[i] - starts[i] for i in range(len(vis)))
    print(f"place late {late:.2f}s early {early:.2f}s")
    end = starts[-1] + durations[-1] + 0.4
    mix = array.array("h", [0]) * int(max(VIDEO, end) * SR)
    peak = 1
    for i, (_dur, text) in enumerate(tokens):
        samples = clips[i]
        at = int(starts[i] * SR)
        for k, s in enumerate(samples):
            j = at + k
            if j >= len(mix):
                break
            v = mix[j] + s
            if v > 32767:
                v = 32767
            elif v < -32768:
                v = -32768
            mix[j] = v
            peak = max(peak, abs(v))
    if peak > 30000:
        scale = 28000 / peak
        for i, s in enumerate(mix):
            mix[i] = int(s * scale)
        print("scaled", peak)
    out = ROOT / "ii3-voice.wav"
    write_wav(out, mix)
    print("wrote", out, "dur", len(mix) / SR, "peak", peak)


if __name__ == "__main__":
    main()
