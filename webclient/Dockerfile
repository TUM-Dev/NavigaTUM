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

FROM nginxinc/nginx-unprivileged:latest as production-stage
COPY nginx.conf  /etc/nginx/conf.d/default.conf
COPY --from=build-stage /app/build /usr/share/nginx/html
EXPOSE 8080