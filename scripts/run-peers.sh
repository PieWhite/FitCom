#!/usr/bin/env bash
# Isolated local peers; PID and command checks avoid killing normal installations.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
peer_root="$root/.localpeers"
count=2
profile=debug
action=start
while (( $# )); do
  case "$1" in
    --stop) action=stop ;;
    --logs) action=logs ;;
    --release) profile=release ;;
    --count) shift; count="${1:?missing count}" ;;
    *) echo "Unknown option: $1" >&2; exit 2 ;;
  esac
  shift
done
[[ "$count" =~ ^[0-9]+$ ]] && (( count >= 2 && count <= 16 )) || exit 2
if [[ "$action" == logs ]]; then
  find "$peer_root" -name '*.log' -type f -exec tail -n 25 {} \;
  exit 0
fi
if [[ -f "$peer_root/processes.txt" ]]; then
  while read -r peer_pid; do
    [[ "$peer_pid" =~ ^[0-9]+$ ]] || exit 2
    args="$(ps -p "$peer_pid" -o args= 2>/dev/null || true)"
    if [[ "$args" == *"fitcom --data-dir $peer_root/peer"* ]]; then kill "$peer_pid" 2>/dev/null || true; fi
  done < "$peer_root/processes.txt"
fi
[[ "$action" == stop ]] && exit 0
cd "$root"
build=(build --locked -p fitcom --bin fitcom)
[[ "$profile" != release ]] || build+=(--release)
cargo "${build[@]}"
[[ "$peer_root" == "$root/.localpeers" && ! -L "$peer_root" ]] || exit 2
rm -rf -- "$peer_root"
mkdir -p "$peer_root"
for ((i=0;i<count;i++)); do
  dir="$peer_root/peer$((i+1))"
  mkdir -p "$dir"
  {
    echo "display_name = \"peer$((i+1))\""
    echo "control_port = $((42650+i*2))"
    echo "media_port = $((42651+i*2))"
    echo 'port_mapping = false'
    for ((j=0;j<count;j++)); do
      if (( j != i )); then
        printf '\n[[peers]]\naddress = "127.0.0.1"\nlabel = "peer%s"\ncontrol_port = %s\n' "$((j+1))" "$((42650+j*2))"
      fi
    done
  } > "$dir/config.toml"
  "$root/target/$profile/fitcom" --data-dir "$dir" >/dev/null 2>&1 &
  echo "$!" >> "$peer_root/processes.txt"
done
