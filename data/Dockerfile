FROM python:3.14-bookworm AS build-stage
WORKDIR /app

ARG GIT_COMMIT_SHA
ENV GIT_COMMIT_SHA=${GIT_COMMIT_SHA}

# install requirements
COPY requirements.txt .
RUN pip3 install -r requirements.txt

# collect data
COPY sources/ sources/
COPY external/ external/
COPY processors/ processors/
COPY *.py ./
COPY translations.yaml translations.yaml

COPY output output

RUN python3 compile.py \
  && test -f "./output/status_data.json" \
  && test -f "./output/status_data.parquet" \
  && test -f "./output/search_data.json" \
  && test -f "./output/search_data.parquet" \
  && test -f "./output/api_data.json" \
  && test -f "./output/alias_data.parquet"

RUN cp -r sources/img/* output \
  && cp -r output/maps/site_plans output/maps/roomfinder \
  && cp -r output/maps/overlays output/maps/overlay \
  && cp external/results/public_transport.parquet output/

# Compress data (only using gzip, because brotli is complex)
RUN gzip --force --keep --recursive output/

# Create final output directory structure for extraction
RUN mkdir -p /cdn && cp -r output/* /cdn/

# This image is meant to be used as a data source in multi-stage builds
# No runtime server is needed - the server image will copy files from /cdn
