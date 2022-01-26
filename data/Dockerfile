FROM ubuntu:latest as build-stage
WORKDIR /app
# install requirements
COPY requirements.txt .
RUN apt-get install python3
RUN pip3 install -r requirements.txt

# collect data
COPY . .
RUN python3 compile.py

# put all data into the cdn folder
RUN mkdir /build
COPY output/* /build/
COPY external/maps/* /build/maps/
COPY sources/img/* /build/img/

FROM nginx as production-stage
RUN mkdir /cdn
COPY --from=build-stage /build /cdn
COPY nginx.conf /etc/nginx/nginx.conf
