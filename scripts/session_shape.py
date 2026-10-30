#!/usr/bin/env python3
"""Compares how Claude Code sessions spend tokens, by the way each was started.

Reads the transcripts under ~/.claude/projects/<folder>/*.jsonl and groups every figure by the
transcript's `entrypoint`: `cli` is a terminal session, `sdk-cli` a headless one (Rekall's chat
view). Read-only, standard library only, and it prints aggregates: no message text, tool input
or path is ever printed.

    scripts/session_shape.py                        every folder under ~/.claude/projects
    scripts/session_shape.py ~/Projects/Me/app      a project folder, or a transcript folder
    scripts/session_shape.py --days 30              tokens per day over the last 30 days

Cost weights follow the price ratios of the four counters (input 1, cache write 1.25, cache read
0.1, output 5) and ignore the price gap between models.
"""

import argparse
import json
from collections import defaultdict
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from pathlib import Path
from statistics import median
from typing import Iterable

COUNTERS = ("input_tokens", "cache_creation_input_tokens", "cache_read_input_tokens", "output_tokens")
COUNTER_LABELS = ("input", "cache write", "cache read", "output")
WEIGHTS = {"input_tokens": 1.0, "cache_creation_input_tokens": 1.25, "cache_read_input_tokens": 0.1, "output_tokens": 5.0}
ENTRYPOINT_NAMES = {"cli": "cli (terminal)", "sdk-cli": "sdk-cli (chat, headless)"}
UNKNOWN_ENTRYPOINT = "unknown"
COLD_SHARE_OF_CONTEXT = 0.5
MINIMUM_COLD_CONTEXT = 10_000
CHECKPOINT_TURNS = (1, 10, 25, 50, 100, 200)
IDLE_GAPS = ((300.0, "up to 5 minutes"), (3600.0, "5 minutes to 1 hour"), (float("inf"), "over 1 hour"))
PROJECTS_FOLDER = Path.home() / ".claude" / "projects"


@dataclass
class Turn:
    time: float
    model: str
    counts: dict[str, int]

    @property
    def context(self) -> int:
        return sum(self.counts[name] for name in COUNTERS[:3])

    @property
    def cache_write(self) -> int:
        return self.counts["cache_creation_input_tokens"]

    @property
    def weighted_cost(self) -> float:
        return sum(self.counts[name] * WEIGHTS[name] for name in COUNTERS)

    @property
    def day(self) -> str:
        return datetime.fromtimestamp(self.time, timezone.utc).strftime("%Y-%m-%d")

    def is_cold(self) -> bool:
        return self.context >= MINIMUM_COLD_CONTEXT and self.cache_write >= COLD_SHARE_OF_CONTEXT * self.context


@dataclass
class Session:
    entrypoint: str
    turns: list[Turn] = field(default_factory=list)


def parse_time(stamp: str | None) -> float | None:
    if not stamp:
        return None
    try:
        return datetime.fromisoformat(stamp.replace("Z", "+00:00")).timestamp()
    except ValueError:
        return None


def entries_of(path: Path) -> Iterable[dict]:
    with path.open(errors="replace") as lines:
        for line in lines:
            try:
                entry = json.loads(line)
            except ValueError:
                continue
            if isinstance(entry, dict):
                yield entry


def counts_of(usage: dict) -> dict[str, int]:
    return {name: int(usage.get(name) or 0) for name in COUNTERS}


def session_of(path: Path) -> Session | None:
    entrypoint = UNKNOWN_ENTRYPOINT
    turns_by_message: dict[str, Turn] = {}
    for entry in entries_of(path):
        if entrypoint == UNKNOWN_ENTRYPOINT and entry.get("entrypoint"):
            entrypoint = str(entry["entrypoint"])
        message = entry.get("message")
        if entry.get("type") != "assistant" or not isinstance(message, dict) or not isinstance(message.get("usage"), dict):
            continue
        time = parse_time(entry.get("timestamp"))
        if time is None:
            continue
        key = message.get("id") or entry.get("uuid") or f"{path.name}:{len(turns_by_message)}"
        counts = counts_of(message["usage"])
        known = turns_by_message.get(key)
        if known is None:
            turns_by_message[key] = Turn(time, str(message.get("model") or "?"), counts)
        else:
            known.counts = {name: max(known.counts[name], counts[name]) for name in COUNTERS}
    turns = sorted((turn for turn in turns_by_message.values() if turn.context + turn.counts["output_tokens"] > 0), key=lambda turn: turn.time)
    return Session(entrypoint, turns) if turns else None


def transcript_folders(arguments: list[str]) -> list[Path]:
    if not arguments:
        return sorted(folder for folder in PROJECTS_FOLDER.glob("*") if folder.is_dir())
    return [folder_of(argument) for argument in arguments]


def folder_of(argument: str) -> Path:
    given = Path(argument).expanduser()
    if any(given.glob("*.jsonl")):
        return given
    mangled = str(given.resolve()).replace("/", "-").replace(".", "-")
    return PROJECTS_FOLDER / mangled


def sessions_in(folders: list[Path]) -> list[Session]:
    sessions = (session_of(path) for folder in folders for path in sorted(folder.glob("*.jsonl")))
    return [session for session in sessions if session]


def by_entrypoint(sessions: list[Session]) -> dict[str, list[Session]]:
    grouped: dict[str, list[Session]] = defaultdict(list)
    for session in sessions:
        grouped[session.entrypoint].append(session)
    return dict(sorted(grouped.items()))


def name_of(entrypoint: str) -> str:
    return ENTRYPOINT_NAMES.get(entrypoint, entrypoint)


def percent(part: float, whole: float) -> str:
    return f"{100 * part / whole:5.1f}%" if whole else "    -"


def all_turns(sessions: list[Session]) -> list[Turn]:
    return [turn for session in sessions for turn in session.turns]


def report_tokens_per_day(groups: dict[str, list[Session]], days: int) -> None:
    since = (datetime.now(timezone.utc) - timedelta(days=days)).strftime("%Y-%m-%d")
    totals: dict[tuple[str, str], list[int]] = defaultdict(lambda: [0, 0, 0, 0, 0])
    for entrypoint, sessions in groups.items():
        for session in sessions:
            for turn in session.turns:
                if turn.day < since:
                    continue
                row = totals[(turn.day, entrypoint)]
                row[0] += 1
                for position, name in enumerate(COUNTERS, start=1):
                    row[position] += turn.counts[name]
    print(f"== tokens per day, last {days} days (UTC days, one row per entrypoint)")
    print(f"   {'day':10}  {'entrypoint':26} {'turns':>6}" + "".join(f" {label:>12}" for label in COUNTER_LABELS))
    for (day, entrypoint), row in sorted(totals.items()):
        print(f"   {day:10}  {name_of(entrypoint):26} {row[0]:6}" + "".join(f" {value:12,}" for value in row[1:]))


def report_cold_turns(groups: dict[str, list[Session]]) -> None:
    print("\n== cold turns: not the session's first, context of at least 10k tokens, half or more of it written to the cache")
    for entrypoint, sessions in groups.items():
        later = [turn for session in sessions for turn in session.turns[1:]]
        cold = [turn for turn in later if turn.is_cold()]
        written = sum(turn.cache_write for turn in all_turns(sessions))
        print(
            f"   {name_of(entrypoint):26} turns {len(later):7,}  cold {len(cold):6,} ({percent(len(cold), len(later)).strip()})"
            f"  share of all cache writes {percent(sum(turn.cache_write for turn in cold), written).strip()}"
        )


def report_context_growth(groups: dict[str, list[Session]]) -> None:
    print("\n== context size at turn N (median over sessions that reached it) and median growth per turn")
    header = "".join(f" {'turn ' + str(n):>9}" for n in CHECKPOINT_TURNS)
    print(f"   {'entrypoint':26}{header}   growth/turn")
    for entrypoint, sessions in groups.items():
        cells = []
        for checkpoint in CHECKPOINT_TURNS:
            reached = [session.turns[checkpoint - 1].context for session in sessions if len(session.turns) >= checkpoint]
            cells.append(f" {int(median(reached)):9,}" if reached else f" {'-':>9}")
        long_sessions = [session for session in sessions if len(session.turns) >= 2]
        growth = [(session.turns[-1].context - session.turns[0].context) / (len(session.turns) - 1) for session in long_sessions]
        print(f"   {name_of(entrypoint):26}{''.join(cells)}   {int(median(growth)):9,}" if growth else f"   {name_of(entrypoint):26}{''.join(cells)}   {'-':>9}")


def report_cold_by_idle_gap(groups: dict[str, list[Session]]) -> None:
    print("\n== share of turns that were cold, by the idle gap before them")
    for entrypoint, sessions in groups.items():
        buckets: dict[str, list[int]] = {label: [0, 0] for _, label in IDLE_GAPS}
        for session in sessions:
            for previous, current in zip(session.turns, session.turns[1:]):
                gap = current.time - previous.time
                label = next(name for limit, name in IDLE_GAPS if gap <= limit)
                buckets[label][0] += 1
                buckets[label][1] += current.is_cold()
        cells = "  ".join(f"{label}: {percent(cold, count).strip()} of {count:,}" for label, (count, cold) in buckets.items())
        print(f"   {name_of(entrypoint):26} {cells}")


def report_models(groups: dict[str, list[Session]]) -> None:
    print("\n== share of weighted cost by model")
    for entrypoint, sessions in groups.items():
        cost_by_model: dict[str, float] = defaultdict(float)
        for turn in all_turns(sessions):
            cost_by_model[turn.model] += turn.weighted_cost
        total = sum(cost_by_model.values())
        shares = "  ".join(f"{model} {percent(cost, total).strip()}" for model, cost in sorted(cost_by_model.items(), key=lambda item: -item[1]))
        print(f"   {name_of(entrypoint):26} {shares}")


def report(sessions: list[Session], days: int) -> None:
    groups = by_entrypoint(sessions)
    summary = ", ".join(f"{len(grouped)} {name_of(entrypoint)}" for entrypoint, grouped in groups.items())
    print(f"{len(sessions)} sessions: {summary}; {len(all_turns(sessions)):,} turns\n")
    report_tokens_per_day(groups, days)
    report_cold_turns(groups)
    report_context_growth(groups)
    report_cold_by_idle_gap(groups)
    report_models(groups)


def main() -> None:
    parser = argparse.ArgumentParser(description="Compare token use of terminal and chat sessions by transcript entrypoint.")
    parser.add_argument("folders", nargs="*", help="project folders or transcript folders; every folder under ~/.claude/projects when none")
    parser.add_argument("--days", type=int, default=14, help="days shown in the tokens-per-day table")
    arguments = parser.parse_args()
    sessions = sessions_in(transcript_folders(arguments.folders))
    if not sessions:
        print("No transcript with a model call was found.")
        return
    report(sessions, arguments.days)


if __name__ == "__main__":
    main()
