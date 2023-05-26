FROM node:latest as build-stage
WORKDIR /app
COPY package*.json ./
RUN npm install &&  \
    apt-get update &&  \
    apt-get install rsync -y

COPY build.sh build.sh
COPY env env
COPY *.json ./
COPY *.ts ./
COPY public public
COPY *.html ./
COPY src src
RUN sh build.sh

FROM nginx:latest as production-stage
COPY nginx.conf /etc/nginx/nginx.conf
RUN mkdir /app && apt update && apt upgrade -y
COPY --from=build-stage /dist /app
EXPOSE 80
