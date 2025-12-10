FROM ghcr.io/maplibre/martin:latest

# -- fonts --
RUN  wget --quiet -O /tmp/roboto-android.zip https://github.com/googlefonts/roboto/releases/download/v2.138/roboto-android.zip && \
    unzip -q /tmp/roboto-android.zip -d "$(dirname "$0")/fonts/" && \
    rm /tmp/roboto-android.zip

# -- sprites --
RUN  wget --quiet -O /tmp/sprites_maki.zip https://github.com/mapbox/maki/zipball/main && \
    mkdir --parents "$(dirname "$0")/sprites/maki/" && \
    unzip -q /tmp/sprites_maki.zip -d /tmp/sprites && \
    rm /tmp/sprites_maki.zip && \
    mv /tmp/sprites/mapbox-maki-*/icons/* "$(dirname "$0")/sprites/maki/" && \
    rm -fr /tmp/sprites/

# -- data --
COPY ./level_overlays/ /map/level_overlays/

# -- styles --
COPY ./styles /map/styles/
COPY ./config.yaml /map/config.yaml

CMD ["martin", "--config", "/map/config.yaml"]
