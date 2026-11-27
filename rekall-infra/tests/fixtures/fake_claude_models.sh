#!/bin/sh
folder=$(dirname "$0")
printf '%s\n' "$@" > "$folder/arguments"
pwd > "$folder/working-directory"
while IFS= read -r line; do
  printf '%s\n' "$line" >> "$folder/received"
  case "$line" in
    *'"subtype":"initialize"'*)
      id=$(printf '%s' "$line" | sed -n 's/.*"request_id":"\([^"]*\)".*/\1/p')
      printf '{"type":"system","subtype":"status"}\n'
      printf '{"type":"control_response","response":{"subtype":"success","request_id":"%s","response":{"commands":[],"models":%s}}}\n' "$id" "$(cat "$folder/models.json")"
      ;;
  esac
done
