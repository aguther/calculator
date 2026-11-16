"""Confirm that a deployment serves the exact web build verified by CI."""
import json
import pathlib
import sys
import time
import urllib.error
import urllib.request


def verify(base_url, expected_build, attempts=12, delay=5):
    last_error = 'No response'
    for attempt in range(attempts):
        try:
            request = urllib.request.Request(
                base_url.rstrip('/') + '/build-info.json',
                headers={'Cache-Control': 'no-cache'},
            )
            with urllib.request.urlopen(request, timeout=10) as response:
                actual = json.loads(response.read(1024 * 1024))['build']
            if actual == expected_build:
                print('Verified deployed build:', actual)
                return
            last_error = 'Expected build ' + expected_build + ', received ' + str(actual)
        except (OSError, ValueError, KeyError, TypeError) as error:
            last_error = str(error)
        if attempt + 1 < attempts:
            time.sleep(delay)
    raise SystemExit('Deployment verification failed: ' + last_error)


if __name__ == '__main__':
    expected = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding='utf-8'))['build']
    verify(sys.argv[1], expected)
