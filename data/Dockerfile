FROM python:3.12-bullseye as build-stage
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
RUN mkdir output \
    && python3 compile.py \
    && test -f "./output/status_data.json" \
    && test -f "./output/status_data.parquet" \
    && test -f "./output/search_data.json" \
    && test -f "./output/search_data.parquet" \
    && test -f "./output/api_data.json" \
    && test -f "./output/api_data.parquet" \
    && cp -r sources/img/* output \
    && cp -r output/img/site_plans output/img/roomfinder \
    && cp -r output/img/overlays output/img/overlay
# last renaming is to remain backwards compatible
COPY output/openapi.yaml output/openapi.yaml

# compress data (only using gzip, because brotli on ngnix is a royal pain)
RUN gzip --force --keep --recursive output/

FROM nginx:1.27 as production-stage
RUN mkdir /cdn
COPY --from=build-stage /app/output /cdn
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 3002
HEALTHCHECK CMD curl --fail http://localhost:3002/cdn/health || exit 1
CMD ["nginx", "-g", "daemon off;"]
