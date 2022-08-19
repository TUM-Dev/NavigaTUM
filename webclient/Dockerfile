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
RUN sed -i "s,    keepalive_timeout  65;,    keepalive_timeout  120;,"  /etc/nginx/nginx.conf && \
    sed -i "s,    sendfile        on;,    sendfile        off;,"  /etc/nginx/nginx.conf && \
    sed -i "s,worker_processes  auto;,worker_processes  1;,"  /etc/nginx/nginx.conf

COPY nginx.conf  /etc/nginx/conf.d/default.conf
COPY --from=build-stage /app/build /usr/share/nginx/html

HEALTHCHECK --start-period=20m  --timeout=10s CMD curl --fail localhost:8080/health || exit 1
EXPOSE 8080
