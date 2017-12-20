#!/bin/bash

section=$1

cat <<EOTOC
# ${section}

## table of contents

EOTOC

for recipe_file in $(find ${section} -name '*.md' -not -name 'README.md'); do
  recipe=$(cat ${recipe_file} | head -n1 | tail -c+3)
  echo "- ${recipe}"
done;

