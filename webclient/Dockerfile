FROM node:latest as build-stage
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY config.js config.js
COPY gulpfile.js gulpfile.js
COPY src src
RUN node_modules/gulp/bin/gulp.js --gulpfile ./gulpfile.js release && rm -fr ./build/tmp

# compress data (only using gzip, because brotli on ngnix is a royal pain)
RUN gzip --force --keep --recursive ./build

FROM nginx:latest as production-stage
COPY nginx.conf /etc/nginx/nginx.conf
RUN mkdir /app && apt update && apt upgrade -y
COPY --from=build-stage /app/build /app
EXPOSE 80
