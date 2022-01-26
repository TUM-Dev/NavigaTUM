FROM node:latest as build-stage
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY ./ .
RUN node_modules/gulp/bin/gulp.js --gulpfile ./gulpfile.js release && rm -fr /app/build/tmp

FROM nginx as production-stage
RUN mkdir /app
COPY --from=build-stage /app/build /app
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 80