FROM python:3.10.2-bullseye as build-stage
WORKDIR /app
# install requirements
COPY requirements.txt .
RUN pip3 install -r requirements.txt

# collect data
ADD . .
RUN rm -fr output && mkdir output
RUN python3 compile.py && test -f "./output/search_data.json" && test -f "./output/api_data.json"

# put all data into the cdn folder
COPY external/maps/* ./output/maps/
COPY sources/img/* ./output/img/

FROM nginx as production-stage
RUN mkdir /cdn
COPY --from=build-stage /app/output /cdn
COPY nginx.conf /etc/nginx/nginx.conf
