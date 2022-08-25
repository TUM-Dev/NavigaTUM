FROM node:latest as build-stage
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
RUN npm run build

# compress data (only using gzip, because brotli on ngnix is a royal pain)
RUN gzip --force --keep --recursive ./dist

FROM nginx as production-stage
COPY nginx.conf /etc/nginx/nginx.conf
RUN mkdir /app && apt update && apt upgrade -y
COPY --from=build-stage /app/dist /app
EXPOSE 80
