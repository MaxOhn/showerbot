#!/bin/sh
set -e

PUID=${PUID:-1000}
PGID=${PGID:-1000}

chown -R "${PUID}:${PGID}" /app/maps

exec gosu "${PUID}:${PGID}" ./showerbot
