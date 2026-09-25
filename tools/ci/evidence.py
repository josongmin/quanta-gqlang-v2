"""Symlink-safe atomic JSON evidence writes under one owned target root."""

from __future__ import annotations

import json
import os
import secrets
import stat
from pathlib import Path
from typing import Tuple


class EvidencePathError(RuntimeError):
    pass


def _open_parent(path: Path, owned_root: Path, *, create: bool) -> int:
    parent = path.parent
    if create:
        parent.mkdir(parents=True, exist_ok=True)
    if parent.is_symlink() or not parent.is_dir():
        raise EvidencePathError("evidence directory is missing or redirected")
    try:
        parent.resolve().relative_to(owned_root.resolve())
    except ValueError as error:
        raise EvidencePathError("evidence directory escapes its owned root") from error

    flags = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0)
    try:
        return os.open(parent, flags)
    except OSError as error:
        raise EvidencePathError("cannot open owned evidence directory") from error


def write_json_evidence(path: Path, payload: dict, *, owned_root: Path, prefix: str) -> None:
    directory_fd = _open_parent(path, owned_root, create=True)
    temporary = ".%s-%s" % (prefix, secrets.token_hex(16))
    try:
        file_fd = os.open(
            temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600, dir_fd=directory_fd
        )
        try:
            with os.fdopen(file_fd, "w", encoding="utf-8") as handle:
                json.dump(payload, handle, indent=2, sort_keys=True)
                handle.write("\n")
                handle.flush()
                os.fsync(handle.fileno())
        except BaseException:
            try:
                os.unlink(temporary, dir_fd=directory_fd)
            except FileNotFoundError:
                pass
            raise
        os.replace(
            temporary, path.name, src_dir_fd=directory_fd, dst_dir_fd=directory_fd
        )
        os.fsync(directory_fd)
    finally:
        try:
            os.unlink(temporary, dir_fd=directory_fd)
        except FileNotFoundError:
            pass
        os.close(directory_fd)


def read_json_evidence(
    path: Path, *, owned_root: Path, consume: bool = False,
) -> Tuple[object, bytes]:
    directory_fd = _open_parent(path, owned_root, create=False)
    file_fd = None
    try:
        flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
        try:
            file_fd = os.open(path.name, flags, dir_fd=directory_fd)
        except OSError as error:
            raise EvidencePathError("evidence file is missing or redirected") from error
        metadata = os.fstat(file_fd)
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
            raise EvidencePathError("evidence file is not an owned regular file")
        if consume:
            os.unlink(path.name, dir_fd=directory_fd)
            os.fsync(directory_fd)
        with os.fdopen(file_fd, "rb") as handle:
            file_fd = None
            raw = handle.read()
        try:
            return json.loads(raw.decode("utf-8")), raw
        except (UnicodeDecodeError, ValueError) as error:
            raise EvidencePathError("evidence file is not valid UTF-8 JSON") from error
    finally:
        if file_fd is not None:
            os.close(file_fd)
        os.close(directory_fd)


def consume_json_evidence(path: Path, expected_raw: bytes, *, owned_root: Path) -> None:
    directory_fd = _open_parent(path, owned_root, create=False)
    file_fd = None
    try:
        flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
        try:
            file_fd = os.open(path.name, flags, dir_fd=directory_fd)
        except OSError as error:
            raise EvidencePathError("evidence file is missing or redirected") from error
        metadata = os.fstat(file_fd)
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
            raise EvidencePathError("evidence file is not an owned regular file")
        with os.fdopen(file_fd, "rb") as handle:
            file_fd = None
            if handle.read() != expected_raw:
                raise EvidencePathError("evidence file changed before consumption")
        current = os.stat(path.name, dir_fd=directory_fd, follow_symlinks=False)
        if (current.st_dev, current.st_ino) != (metadata.st_dev, metadata.st_ino):
            raise EvidencePathError("evidence file changed before consumption")
        os.unlink(path.name, dir_fd=directory_fd)
        os.fsync(directory_fd)
    finally:
        if file_fd is not None:
            os.close(file_fd)
        os.close(directory_fd)
