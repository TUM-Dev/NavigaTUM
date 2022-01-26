FROM python:3.9.10-bullseye as build-stage
WORKDIR /app
# install requirements
COPY requirements.txt .
RUN pip3 install -r requirements.txt

# collect data
ADD . .
RUN ls -a .
RUN rm ./output/*
RUN python3 compile.py && test -f "./output/search_data.json" && test -f "./output/api_data.json"
RUN ls -a ./output

# put all data into the cdn folder
RUN mkdir /build
COPY output/* /build/
COPY external/maps/* /build/maps/
COPY sources/img/* /build/img/

FROM nginx as production-stage
RUN mkdir /cdn
COPY --from=build-stage /build /cdn
COPY nginx.conf /etc/nginx/nginx.conf
