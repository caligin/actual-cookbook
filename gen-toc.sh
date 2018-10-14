#!/bin/bash

baseurl=https://github.com/caligin/actual-cookbook/tree/master/
cat README.prefix.md

for section in $(find . -mindepth 2 -name README.md | sort | xargs dirname); do
  echo "### $(echo ${section} | tail -c+3)"
  echo
  for recipe_file in $(find ${section} -mindepth 1 -name '*.md' -not -name 'README.md' | sort); do
    recipe=$(cat ${recipe_file} | head -n1 | tail -c+3)
    echo "- [${recipe}](${baseurl}$(echo ${recipe_file} | tail -c+3))"
  done;
  echo
done;

