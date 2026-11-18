#!/bin/sh
# Daily Postgres backup, run by the `backup` service in docker-compose.yml.
# Keeps the 7 most recent daily dumps and the 4 most recent weekly ones
# (Sunday's dump is also copied to weekly/). Dumps use pg_dump's custom
# format: restore with `pg_restore -d <db> <file>`.
set -eu

DAILY=/backups/daily
WEEKLY=/backups/weekly
mkdir -p "$DAILY" "$WEEKLY"

keep_newest() { # dir count
  ls -1t "$1"/*.dump 2>/dev/null | tail -n +"$(($2 + 1))" | while read -r old; do
    rm -f -- "$old"
  done
}

while true; do
  stamp="$(date -u +%Y-%m-%dT%H%M%SZ)"
  file="$DAILY/rpg-$stamp.dump"
  if pg_dump -h db -U "$POSTGRES_USER" -d "$POSTGRES_DB" -Fc -f "$file.partial"; then
    mv "$file.partial" "$file"
    echo "backup ok: $file"
    [ "$(date -u +%u)" = 7 ] && cp "$file" "$WEEKLY/"
    keep_newest "$DAILY" 7
    keep_newest "$WEEKLY" 4
  else
    rm -f "$file.partial"
    echo "backup FAILED at $stamp" >&2
  fi
  sleep 86400
done
