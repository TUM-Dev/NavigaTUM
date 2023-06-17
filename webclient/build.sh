#!/usr/bin/env sh

set -e # fail on first error

mkdir -p dist
rm -fr dist

for LANG in en de
do
  for THEME in light dark
  do
    # make sure we are really only building the right theme and language
    sed -i "s/\$theme: .*/\$theme: \"${THEME}\";/" src/assets/variables.scss
    sed -i "s/locale: .*/locale: \"${LANG}\",/" src/main.ts
    sed -i "/fallbackLocale: .*/d" src/main.ts
    sed -i "s/messages: .*/messages: { ${LANG} },/" src/main.ts

    echo "Building ${LANG}-${THEME}"
    npm run build-only
    mv dist/index.html dist/${LANG}-${THEME}.html
    rsync -r dist/* ../dist
  done
done


# compress data (only using gzip, because brotli on ngnix is a royal pain)
gzip --force --keep --recursive ../dist
