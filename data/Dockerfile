FROM python:3.13-bookworm AS build-stage
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
    && test -f "./output/api_data.parquet"
RUN cp -r sources/img/* output \
    && cp -r output/maps/site_plans output/maps/roomfinder \
    && cp -r output/maps/overlays output/maps/overlay
# last renaming is to remain backwards compatible

# compress data (only using gzip, because brotli on ngnix is a royal pain)
RUN gzip --force --keep --recursive output/

FROM nginx:1.27 AS production-stage
RUN mkdir /cdn
COPY --from=build-stage /app/output /cdn
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 3002
HEALTHCHECK CMD curl --fail http://localhost:3002/cdn/health || exit 1
CMD ["nginx", "-g", "daemon off;"]
