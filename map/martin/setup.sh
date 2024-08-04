#!/bin/bash

sudo rm -rf $(dirname "$0")/fonts $(dirname "$0")/sprites

# fonts
wget -q -O /tmp/roboto-android.zip https://github.com/googlefonts/roboto/releases/download/v2.138/roboto-android.zip
unzip /tmp/roboto-android.zip -d $(dirname "$0")/fonts/
rm /tmp/roboto-android.zip

wget -q -O /tmp/open-sans.zip https://www.1001fonts.com/download/open-sans.zip
unzip /tmp/open-sans.zip -d $(dirname "$0")/fonts/
rm /tmp/open-sans.zip
mv "$(dirname "$0")/fonts/OpenSans-Bold.ttf" "$(dirname "$0")/fonts/OpenSans Bold.ttf"
mv "$(dirname "$0")/fonts/OpenSans-BoldItalic.ttf" "$(dirname "$0")/fonts/OpenSans BoldItalic.ttf"
mv "$(dirname "$0")/fonts/OpenSans-Italic.ttf" "$(dirname "$0")/fonts/OpenSans Italic.ttf"
mv "$(dirname "$0")/fonts/OpenSans-ExtraBold.ttf" "$(dirname "$0")/fonts/OpenSans ExtraBold.ttf"
mv "$(dirname "$0")/fonts/OpenSans-ExtraBoldItalic.ttf" "$(dirname "$0")/fonts/OpenSans ExtraBoldItalic.ttf"
mv "$(dirname "$0")/fonts/OpenSans-Light.ttf" "$(dirname "$0")/fonts/OpenSans Light.ttf"
mv "$(dirname "$0")/fonts/OpenSans-LightItalic.ttf" "$(dirname "$0")/fonts/OpenSans LightItalic.ttf"
mv "$(dirname "$0")/fonts/OpenSans-Regular.ttf" "$(dirname "$0")/fonts/OpenSans Regular.ttf"
mv "$(dirname "$0")/fonts/OpenSans-Semibold.ttf" "$(dirname "$0")/fonts/OpenSans Semibold.ttf"
mv "$(dirname "$0")/fonts/OpenSans-SemiboldItalic.ttf" "$(dirname "$0")/fonts/OpenSans SemiboldItalic.ttf"

wget -q -O "$(dirname "$0")/fonts/Arial Unicode Bold Italic.ttf" https://github.com/stamen/toner-carto/raw/master/fonts/Arial-Unicode-Bold-Italic.ttf
wget -q -O "$(dirname "$0")/fonts/Arial Unicode Bold.ttf" https://github.com/stamen/toner-carto/raw/master/fonts/Arial-Unicode-Bold.ttf
wget -q -O "$(dirname "$0")/fonts/Arial Unicode Italic.ttf" https://github.com/stamen/toner-carto/raw/master/fonts/Arial-Unicode-Italic.ttf
wget -q -O "$(dirname "$0")/fonts/Arial Unicode Regular.ttf" https://github.com/stamen/toner-carto/raw/master/fonts/Arial-Unicode-Regular.ttf

# sprites
wget -q -O /tmp/sprites_maki.zip https://github.com/mapbox/maki/zipball/main
unzip -q /tmp/sprites_maki.zip -d /tmp/sprites/
rm /tmp/sprites_maki.zip
mkdir $(dirname "$0")/sprites/
mv /tmp/sprites/mapbox-maki-*/icons/* $(dirname "$0")/sprites/
rm -fr /tmp/sprites/
