#!/usr/bin/env sh
# Rolls the production stack to the images tagged with the given commit
# sha: records it as IMAGE_TAG in .env (and IMAGE_NAMESPACE when the
# workflow passes one, so a later manual `up -d` keeps the same version),
# pulls, recreates the changed containers and waits for the api to
# answer. Run by the deploy workflow over ssh after it has copied the
# compose files next to this script and logged the box into GHCR; usable
# by hand for a rollback: `deploy/deploy.sh <older sha>` (log in to
# ghcr.io first).
set -eu
cd "$(dirname "$0")/.."

tag="${1:?usage: deploy/deploy.sh <image tag>}"
compose="docker compose -f docker-compose.yml -f docker-compose.prod.yml"

set_env() {
  if grep -q "^$1=" .env; then
    sed -i "s|^$1=.*|$1=$2|" .env
  else
    printf '\n%s=%s\n' "$1" "$2" >> .env
  fi
}

set_env IMAGE_TAG "$tag"
if [ -n "${IMAGE_NAMESPACE:-}" ]; then
  set_env IMAGE_NAMESPACE "$IMAGE_NAMESPACE"
fi

$compose pull --quiet
$compose up -d --remove-orphans

health_attempts=60
origin="$(sed -n 's/^APP_URL=//p' .env)"
i=0
until curl -fsS --max-time 5 "$origin/api/health" > /dev/null; do
  i=$((i + 1))
  if [ "$i" -ge "$health_attempts" ]; then
    echo "api did not become healthy at $origin/api/health" >&2
    $compose logs --tail 50 api >&2
    exit 1
  fi
  sleep 5
done
echo "deployed $tag"

# Superseded sha-tagged images would otherwise accumulate on the disk.
docker image prune -f > /dev/null
