import subprocess
import sys
from pathlib import Path


# ----------------------------
# Hardcoded settings
# ----------------------------
ROOT_DIRECTORY = r"examples\Synapse"
FILE_EXTENSION = ".json"
DOXCER_EXE = "doxcer.exe"
DOXCER_SELECTOR = "-synapse"


def main():
    root_path = Path(ROOT_DIRECTORY)

    if not root_path.exists():
        print(f"Root directory does not exist: {root_path}")
        sys.exit(1)

    pattern = f"*{FILE_EXTENSION}"
    files = sorted(root_path.rglob(pattern))

    print(f"Found {len(files)} file(s) with extension '{FILE_EXTENSION}' in:")
    print(f"  {root_path}\n")

    if not files:
        return

    for i, file_path in enumerate(files, start=1):
        command = [DOXCER_EXE]
        if DOXCER_SELECTOR.strip():
            command.append(DOXCER_SELECTOR.strip())
        command.append(str(file_path))

        print(f"[{i}/{len(files)}] Running:")
        print(f"  {' '.join(command)}")

        proc = subprocess.run(command)
        if proc.returncode != 0:
            print(f"\nDoxcer failed (exit code {proc.returncode}) on:")
            print(f"  {file_path}")
            sys.exit(proc.returncode)

    print("\nAll files processed successfully.")


if __name__ == "__main__":
    main()