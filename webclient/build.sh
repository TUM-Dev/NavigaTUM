#!/usr/bin/env sh

set -e # fail on first error

mkdir -p ../dist
rm -fr ../dist

for THEME in light dark
do
  # make sure we are really only building the right theme
  sed -i "s/\$theme: .*/\$theme: \"${THEME}\";/" src/assets/variables.scss
  sed -i "s/class='light'/${THEME}/" index.html

  echo "Building ${THEME}"
  npm run build-only
  mv dist/index.html dist/${THEME}.html
  rsync -r dist/* ../dist
done


# compress data (only using gzip, because brotli on ngnix is a royal pain)
gzip --force --keep --recursive ../dist
