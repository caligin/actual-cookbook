#!/bin/bash

cat README.prefix.md

for section in $(find . -mindepth 2 -name README.md | xargs dirname); do
  echo "### $(echo ${section} | tail -c+3)"
  echo
  for recipe_file in $(find ${section} -mindepth 1 -name '*.md' -not -name 'README.md'); do
    recipe=$(cat ${recipe_file} | head -n1 | tail -c+3)
    echo "- ${recipe}"
  done;
  echo
done;

