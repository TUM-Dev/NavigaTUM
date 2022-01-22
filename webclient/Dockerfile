FROM node:latest as build-stage
WORKDIR /app
COPY package*.json ./
RUN npm install
RUN npm install -g gulp
RUN npm install gulp
RUN npm install -g --force yarn
RUN npm install yarn
COPY ./ .
RUN gulp release

FROM nginx as production-stage
RUN mkdir /app
COPY --from=build-stage /app/build /app
#RUN chown nginx:nginx ./*
COPY nginx.conf /etc/nginx/nginx.conf
