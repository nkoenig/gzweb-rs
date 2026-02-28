#!/bin/bash

REPOS=(
  "gz-cmake gz-cmake3"
  "gz-utils gz-utils2"
  "gz-tools gz-tools2"
  "gz-plugin gz-plugin2"
  "gz-math gz-math7"
  "gz-common gz-common5"
  "sdformat sdf14"
  "gz-msgs gz-msgs10"
  "gz-transport gz-transport13"
  "gz-fuel-tools gz-fuel-tools9"
  "gz-physics gz-physics7"
  "gz-rendering gz-rendering8"
  "gz-sensors gz-sensors8"
  "gz-gui gz-gui8"
  "gz-sim gz-sim8"
  "gz-launch gz-launch7"
)

build_repo() {
  local repo="$1"
  local branch="$2"

  echo "Processing repo: $repo  branch: $branch"

  if [ -d "$repo" ]; then
    cd "$repo"
    git fetch
    git checkout "$branch"
    git pull origin "$branch"
  else
    git clone git@github.com:gazebosim/"$repo" -b "$branch"
    cd "$repo"
  fi

  mkdir -p build
  cd build
  cmake -DCMAKE_INSTALL_PREFIX=/home/nate/local -DCMAKE_BUILD_TYPE=RelWithDebInfo ../; make -j10
  make install
  cd ../../
}

for repo_info in "${REPOS[@]}"; do
  read -r repo branch <<< "$repo_info"
  build_repo "$repo" "$branch"
done
