FROM python:3.11-bullseye as build-stage
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
    && test -f "./output/search_data.json" \
    && test -f "./output/api_data.json" \
    && cp -r sources/img/* output
COPY output/openapi.yaml output/openapi.yaml

# put all data into the cdn folder
COPY external/results/maps/roomfinder output/maps/roomfinder

# compress data (only using gzip, because brotli on ngnix is a royal pain)
RUN gzip --force --keep --recursive output/

FROM nginx:1.25 as production-stage
RUN mkdir /cdn
COPY --from=build-stage /app/output /cdn
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 8000
HEALTHCHECK CMD curl --fail localhost:8000/cdn/health || exit 1
CMD ["nginx", "-g", "daemon off;"]
