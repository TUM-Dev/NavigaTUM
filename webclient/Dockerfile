FROM node:latest as build-stage
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY ./ .
RUN node_modules/gulp/bin/gulp.js --gulpfile ./gulpfile.js release && rm -fr ./build/tmp

# compress data (only using gzip, because brotli on ngnix is a royal pain)
RUN gzip --force --keep --recursive ./build

FROM nginx as production-stage
RUN mkdir /app
COPY --from=build-stage /app/build /app
COPY nginx.conf /etc/nginx/nginx.conf
RUN apt update && apt upgrade -y
EXPOSE 80